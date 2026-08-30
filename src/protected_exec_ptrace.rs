//! Linux ptrace-based exec supervision (ADR 0086).
//!
//! A supervised root command installs `SECCOMP_RET_TRACE` for `execve` and
//! `execveat`, writes its PID through a pre-opened pipe, and stops before
//! `exec`. One tracer thread seizes each root and auto-attaches descendants.
//! The shared functional policy still owns allow, deny, and promotion meaning.
#![cfg(target_os = "linux")]

mod linux {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io;
    use std::io::Read;
    use std::os::fd::AsRawFd;
    use std::os::fd::FromRawFd;
    use std::os::fd::OwnedFd;
    use std::os::fd::RawFd;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::FileExt;
    use std::os::unix::process::CommandExt;
    use std::os::unix::process::ExitStatusExt;
    use std::path::Component;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;
    use std::process::ExitStatus;
    use std::process::Output;
    use std::process::Stdio;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::RwLock;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    use crate::protected_exec::ExecRequest;
    use crate::protected_exec::OutputPromotionRecord;
    use crate::protected_exec::PHASE_PROTECTED;
    use crate::protected_exec::PlannedProducedExecutableRoot;
    use crate::protected_exec::PromotedExecutable;
    use crate::protected_exec::ProtectedExecError;
    use crate::protected_exec::ProtectedExecPolicy;
    use crate::protected_exec::ProtectedSeccompAuditEvent;
    use crate::protected_exec::blake3_file_hex;

    const AUDIT_ARCH_X86_64: u32 = 0xC000_003E;
    const AUDIT_ARCH_AARCH64: u32 = 0xC000_00B7;
    const FILTER_INSTRUCTION_COUNT: usize = 8;
    const FILTER_EXECVE_INDEX: usize = 4;
    const FILTER_EXECVEAT_INDEX: usize = 5;
    const FILTER_TRACE_INDEX: usize = 6;
    const FILTER_ALLOW_INDEX: usize = 7;
    const SECCOMP_DATA_NR_OFFSET: u32 = 0;
    const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
    const SECCOMP_RET_TRACE: u32 = 0x7FF0_0000;
    const SECCOMP_RET_ALLOW: u32 = 0x7FFF_0000;
    const SECCOMP_SET_MODE_FILTER: libc::c_long = 1;
    const SYS_SECCOMP: libc::c_long = 317;
    const PTRACE_SETOPTIONS: libc::c_uint = 0x4200;
    const PTRACE_GETEVENTMSG: libc::c_uint = 0x4201;
    const PTRACE_SEIZE: libc::c_uint = 0x4206;
    const PTRACE_CONT: libc::c_uint = 7;
    const PTRACE_GETREGS: libc::c_uint = 12;
    const PTRACE_SETREGS: libc::c_uint = 13;
    const PTRACE_O_TRACESYSGOOD: libc::c_uint = 1;
    const PTRACE_O_TRACEFORK: libc::c_uint = 2;
    const PTRACE_O_TRACEVFORK: libc::c_uint = 4;
    const PTRACE_O_TRACECLONE: libc::c_uint = 8;
    const PTRACE_O_TRACEEXEC: libc::c_uint = 16;
    const PTRACE_O_TRACESECCOMP: libc::c_uint = 0x80;
    const PTRACE_EVENT_FORK: u32 = 1;
    const PTRACE_EVENT_VFORK: u32 = 2;
    const PTRACE_EVENT_CLONE: u32 = 3;
    const PTRACE_EVENT_SECCOMP: u32 = 7;
    const PTRACE_EVENT_STOP: u32 = 128;
    const STOP_LOW_BYTE_MARKER: u32 = 0x7F;
    const SIGTRAP_BYTE: u32 = libc::SIGTRAP as u32;
    const SYSCALL_STOP_SIGTRAP_BYTE: u32 = libc::SIGTRAP as u32 | 0x80;
    const EXECVE_NR: u64 = 59;
    const EXECVEAT_NR: u64 = 322;
    const AT_FDCWD: i64 = -100;
    const DENIED_SYSCALL_MARKER: u64 = u64::MAX;
    const DENIED_RETURN: u64 = (-(libc::EACCES as i64)) as u64;
    const MAX_REMOTE_PATH_BYTES: usize = 4096;
    const TRACER_POLL_TIMEOUT_MS: libc::c_int = 1;
    const TRACER_LOOP_SLEEP_MS: u64 = 1;
    const TRACEES_MAX: usize = 65_536;
    const QUIESCENCE_POLL_COUNT_MAX: u32 = 6_000;
    const QUIESCENCE_STABLE_POLL_COUNT: u32 = 2;
    const QUIESCENCE_POLL_INTERVAL_MS: u64 = 5;
    const PID_BYTES: usize = std::mem::size_of::<libc::pid_t>();
    const PIPE_FD_COUNT: usize = 2;
    const PIPE_READ_INDEX: usize = 0;
    const PIPE_WRITE_INDEX: usize = 1;

    #[derive(Debug, Eq, PartialEq)]
    pub enum PtraceSupervisorError {
        Unsupported(String),
        Install(String),
        Attach(String),
        TracerLoop(String),
        Policy(String),
    }

    impl std::fmt::Display for PtraceSupervisorError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unsupported(message) => write!(f, "ptrace supervisor unsupported: {message}"),
                Self::Install(message) => write!(f, "installing ptrace supervision failed: {message}"),
                Self::Attach(message) => write!(f, "attaching ptrace supervisor failed: {message}"),
                Self::TracerLoop(message) => write!(f, "ptrace tracer loop failed: {message}"),
                Self::Policy(message) => write!(f, "ptrace supervisor policy failed: {message}"),
            }
        }
    }

    impl std::error::Error for PtraceSupervisorError {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ExecStop {
        Execve { pid: i32, path_address: u64 },
        Execveat { pid: i32, path_address: u64 },
    }

    impl ExecStop {
        fn pid(self) -> i32 {
            match self {
                Self::Execve { pid, .. } | Self::Execveat { pid, .. } => pid,
            }
        }

        fn path_address(self) -> u64 {
            match self {
                Self::Execve { path_address, .. } | Self::Execveat { path_address, .. } => path_address,
            }
        }

        fn syscall_name(self) -> &'static str {
            match self {
                Self::Execve { .. } => "execve",
                Self::Execveat { .. } => "execveat",
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct EntryRegs {
        syscall_nr: u64,
        arg0: u64,
        arg1: u64,
    }

    struct ExecTarget {
        tracee_path: PathBuf,
        resolved_host_path: PathBuf,
    }

    struct SupervisorDecision {
        allowed: bool,
        technical_failure: bool,
        audit_event: ProtectedSeccompAuditEvent,
        promotion: Option<OutputPromotionRecord>,
    }

    struct BpfStatement {
        code: u32,
        operand: u32,
    }

    struct BpfJump {
        code: u32,
        operand: u32,
        jump_true: u8,
        jump_false: u8,
    }

    #[derive(Debug)]
    pub struct ProtectedPtraceSupervisor {
        // Lock order: shared_policy -> audit_events -> auto_promotions.
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: Arc<Mutex<Vec<OutputPromotionRecord>>>,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        fatal_error: Arc<Mutex<Option<String>>>,
        active_tracees: Arc<AtomicUsize>,
        completed_roots: Arc<Mutex<BTreeMap<i32, i32>>>,
        pid_writer: OwnedFd,
    }

    impl ProtectedPtraceSupervisor {
        pub fn audit_events(&self) -> Vec<ProtectedSeccompAuditEvent> {
            match self.audit_events.lock() {
                Ok(events) => events.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }

        pub fn automatic_promotions(&self) -> Vec<OutputPromotionRecord> {
            match self.auto_promotions.lock() {
                Ok(promotions) => promotions.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }

        pub fn register_planned_produced_roots(
            &self,
            roots: &[PlannedProducedExecutableRoot],
        ) -> Result<(), ProtectedExecError> {
            let mut policy = self.shared_policy.write().map_err(|_| ProtectedExecError::PolicyLockPoisoned)?;
            policy.register_planned_produced_roots(roots)
        }

        pub fn begin_producer_action(&self, producer_action_id: &str) -> Result<(), ProtectedExecError> {
            let mut policy = self.shared_policy.write().map_err(|_| ProtectedExecError::PolicyLockPoisoned)?;
            policy.begin_producer_action(producer_action_id)
        }

        pub fn end_producer_action(&self, producer_action_id: &str) -> Result<(), ProtectedExecError> {
            let mut policy = self.shared_policy.write().map_err(|_| ProtectedExecError::PolicyLockPoisoned)?;
            policy.end_producer_action(producer_action_id)
        }

        pub fn promote_verified_output(
            &self,
            source_entry_id: &str,
            extraction_rules: &[String],
            executables: &[PromotedExecutable],
        ) -> Result<OutputPromotionRecord, ProtectedExecError> {
            let mut policy = self.shared_policy.write().map_err(|_| ProtectedExecError::PolicyLockPoisoned)?;
            policy.promote_verified_output(source_entry_id, extraction_rules, executables)
        }

        /// Adds the race-free child half of ADR 0086 to a root command.
        /// Descendants inherit the filter and are attached by ptrace options.
        pub fn prepare_command(&self, command: &mut Command) -> Result<(), PtraceSupervisorError> {
            self.require_healthy()?;
            let pid_writer = self.pid_writer.as_raw_fd();
            // SAFETY: the closure calls only bounded libc operations before exec.
            unsafe {
                command.pre_exec(move || {
                    pre_exec_install_trace_filter().map_err(io::Error::other)?;
                    pre_exec_write_pid(pid_writer)?;
                    pre_exec_raise_stop()?;
                    Ok(())
                });
            }
            Ok(())
        }

        pub fn status(&self, command: &mut Command) -> io::Result<ExitStatus> {
            self.prepare_command(command).map_err(ptracer_io_error)?;
            let child = command.spawn()?;
            let pid = i32::try_from(child.id()).map_err(|_| io::Error::other("child pid exceeds i32"))?;
            drop(child);
            let raw_status = self.wait_for_root_status(pid)?;
            Ok(ExitStatus::from_raw(raw_status))
        }

        pub fn output(&self, command: &mut Command) -> io::Result<Output> {
            self.prepare_command(command).map_err(ptracer_io_error)?;
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
            let mut child = command.spawn()?;
            let pid = i32::try_from(child.id()).map_err(|_| io::Error::other("child pid exceeds i32"))?;
            let mut stdout = child.stdout.take().ok_or_else(|| io::Error::other("child stdout pipe is missing"))?;
            let mut stderr = child.stderr.take().ok_or_else(|| io::Error::other("child stderr pipe is missing"))?;
            let stdout_reader = thread::spawn(move || {
                let mut bytes = Vec::new();
                stdout.read_to_end(&mut bytes).map(|_| bytes)
            });
            let stderr_reader = thread::spawn(move || {
                let mut bytes = Vec::new();
                stderr.read_to_end(&mut bytes).map(|_| bytes)
            });
            drop(child);
            let raw_status = self.wait_for_root_status(pid)?;
            let stdout = stdout_reader.join().map_err(|_| io::Error::other("child stdout reader panicked"))??;
            let stderr = stderr_reader.join().map_err(|_| io::Error::other("child stderr reader panicked"))??;
            Ok(Output {
                status: ExitStatus::from_raw(raw_status),
                stdout,
                stderr,
            })
        }

        pub fn wait_for_audit_quiescence(&self) -> Result<usize, PtraceSupervisorError> {
            let mut previous_count = usize::MAX;
            let mut stable_poll_count = 0_u32;
            for _ in 0..QUIESCENCE_POLL_COUNT_MAX {
                self.require_healthy()?;
                let event_count = self.audit_events().len();
                let active_tracees = self.active_tracees.load(Ordering::Acquire);
                if active_tracees == 0 && event_count == previous_count {
                    stable_poll_count = stable_poll_count.saturating_add(1);
                } else {
                    stable_poll_count = 0;
                    previous_count = event_count;
                }
                if stable_poll_count >= QUIESCENCE_STABLE_POLL_COUNT {
                    return Ok(event_count);
                }
                thread::sleep(Duration::from_millis(QUIESCENCE_POLL_INTERVAL_MS));
            }
            Err(PtraceSupervisorError::TracerLoop(format!(
                "exec audit did not become quiescent; active_tracees={} events={}",
                self.active_tracees.load(Ordering::Acquire),
                self.audit_events().len()
            )))
        }

        fn wait_for_root_status(&self, pid: i32) -> io::Result<i32> {
            loop {
                self.require_healthy().map_err(ptracer_io_error)?;
                let completed = match self.completed_roots.lock() {
                    Ok(mut completed) => completed.remove(&pid),
                    Err(poisoned) => poisoned.into_inner().remove(&pid),
                };
                if let Some(status) = completed {
                    return Ok(status);
                }
                thread::sleep(Duration::from_millis(TRACER_LOOP_SLEEP_MS));
            }
        }

        fn require_healthy(&self) -> Result<(), PtraceSupervisorError> {
            let error = match self.fatal_error.lock() {
                Ok(error) => error.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            };
            match error {
                Some(message) => Err(PtraceSupervisorError::TracerLoop(message)),
                None => Ok(()),
            }
        }
    }

    pub fn install_exec_supervisor(
        policy: ProtectedExecPolicy,
    ) -> Result<ProtectedPtraceSupervisor, PtraceSupervisorError> {
        require_supported_audit_arch(current_audit_arch())?;
        set_child_subreaper()?;
        let (pid_reader, pid_writer) = create_pid_pipe()?;
        let audit_events = Arc::new(Mutex::new(Vec::new()));
        let auto_promotions = Arc::new(Mutex::new(Vec::new()));
        let shared_policy = Arc::new(RwLock::new(policy));
        let fatal_error = Arc::new(Mutex::new(None));
        let active_tracees = Arc::new(AtomicUsize::new(0));
        let completed_roots = Arc::new(Mutex::new(BTreeMap::new()));
        spawn_tracer_thread(
            pid_reader,
            shared_policy.clone(),
            audit_events.clone(),
            auto_promotions.clone(),
            fatal_error.clone(),
            active_tracees.clone(),
            completed_roots.clone(),
        )?;
        Ok(ProtectedPtraceSupervisor {
            audit_events,
            auto_promotions,
            shared_policy,
            fatal_error,
            active_tracees,
            completed_roots,
            pid_writer,
        })
    }

    fn bpf_stmt(statement: BpfStatement) -> libc::sock_filter {
        libc::sock_filter {
            code: statement.code as u16,
            jt: 0,
            jf: 0,
            k: statement.operand,
        }
    }

    fn bpf_jump(jump: BpfJump) -> libc::sock_filter {
        libc::sock_filter {
            code: jump.code as u16,
            jt: jump.jump_true,
            jf: jump.jump_false,
            k: jump.operand,
        }
    }

    fn exec_trace_filter(arch: u32) -> [libc::sock_filter; FILTER_INSTRUCTION_COUNT] {
        assert_ne!(arch, 0);
        [
            bpf_stmt(BpfStatement {
                code: libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                operand: SECCOMP_DATA_ARCH_OFFSET,
            }),
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: arch,
                jump_true: 1,
                jump_false: 0,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: libc::SECCOMP_RET_KILL_PROCESS,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                operand: SECCOMP_DATA_NR_OFFSET,
            }),
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: EXECVE_NR as u32,
                jump_true: 1,
                jump_false: 0,
            }),
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: EXECVEAT_NR as u32,
                jump_true: 0,
                jump_false: 1,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: SECCOMP_RET_TRACE,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: SECCOMP_RET_ALLOW,
            }),
        ]
    }

    fn current_audit_arch() -> Option<u32> {
        if cfg!(target_arch = "x86_64") {
            return Some(AUDIT_ARCH_X86_64);
        }
        if cfg!(target_arch = "aarch64") {
            return Some(AUDIT_ARCH_AARCH64);
        }
        None
    }

    fn require_supported_audit_arch(arch: Option<u32>) -> Result<u32, PtraceSupervisorError> {
        if !cfg!(target_arch = "x86_64") {
            return Err(PtraceSupervisorError::Unsupported(
                "register decoding is implemented only for x86_64".to_string(),
            ));
        }
        arch.ok_or_else(|| {
            PtraceSupervisorError::Unsupported("unsupported Linux audit architecture for exec supervisor".to_string())
        })
    }

    fn set_child_subreaper() -> Result<(), PtraceSupervisorError> {
        let result = unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) };
        if result == 0 {
            return Ok(());
        }
        Err(PtraceSupervisorError::Install(last_error("PR_SET_CHILD_SUBREAPER")))
    }

    fn create_pid_pipe() -> Result<(OwnedFd, OwnedFd), PtraceSupervisorError> {
        const { assert!(PIPE_FD_COUNT == 2) };
        const { assert!(PID_BYTES == 4) };
        let mut fds = [-1; PIPE_FD_COUNT];
        let result = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        if result != 0 {
            return Err(PtraceSupervisorError::Install(last_error("pipe2")));
        }
        // SAFETY: pipe2 returned two new descriptors owned by this function.
        let reader = unsafe { OwnedFd::from_raw_fd(fds[PIPE_READ_INDEX]) };
        // SAFETY: pipe2 returned two new descriptors owned by this function.
        let writer = unsafe { OwnedFd::from_raw_fd(fds[PIPE_WRITE_INDEX]) };
        set_nonblocking(reader.as_raw_fd())?;
        Ok((reader, writer))
    }

    fn set_nonblocking(fd: RawFd) -> Result<(), PtraceSupervisorError> {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 {
            return Err(PtraceSupervisorError::Install(last_error("fcntl F_GETFL")));
        }
        let result = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
        if result != 0 {
            return Err(PtraceSupervisorError::Install(last_error("fcntl F_SETFL")));
        }
        Ok(())
    }

    fn pre_exec_install_trace_filter() -> Result<(), PtraceSupervisorError> {
        let result = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1u64, 0u64, 0u64, 0u64) };
        if result != 0 {
            return Err(PtraceSupervisorError::Install(last_error("PR_SET_NO_NEW_PRIVS")));
        }
        let arch = require_supported_audit_arch(current_audit_arch())?;
        let mut filter = exec_trace_filter(arch);
        let filter_len = u16::try_from(filter.len())
            .map_err(|_| PtraceSupervisorError::Install("filter length exceeds u16".to_string()))?;
        let mut program = libc::sock_fprog {
            len: filter_len,
            filter: filter.as_mut_ptr(),
        };
        let result = unsafe {
            libc::syscall(
                SYS_SECCOMP,
                SECCOMP_SET_MODE_FILTER,
                0 as libc::c_ulong,
                &mut program as *mut libc::sock_fprog,
            )
        };
        if result != 0 {
            return Err(PtraceSupervisorError::Install(last_error("SECCOMP_SET_MODE_FILTER")));
        }
        Ok(())
    }

    fn pre_exec_write_pid(pid_writer: RawFd) -> io::Result<()> {
        let pid = unsafe { libc::getpid() };
        let bytes = pid.to_ne_bytes();
        let result = unsafe { libc::write(pid_writer, bytes.as_ptr().cast::<libc::c_void>(), bytes.len()) };
        if result == PID_BYTES as isize {
            return Ok(());
        }
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        Err(io::Error::other("ptrace PID pipe write was short"))
    }

    fn pre_exec_raise_stop() -> io::Result<()> {
        let result = unsafe { libc::raise(libc::SIGSTOP) };
        if result == 0 {
            return Ok(());
        }
        Err(io::Error::last_os_error())
    }

    fn spawn_tracer_thread(
        pid_reader: OwnedFd,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: Arc<Mutex<Option<String>>>,
        active_tracees: Arc<AtomicUsize>,
        completed_roots: Arc<Mutex<BTreeMap<i32, i32>>>,
    ) -> Result<(), PtraceSupervisorError> {
        thread::Builder::new()
            .name("mantle-ptrace-exec-supervisor".to_string())
            .spawn(move || {
                tracer_loop(
                    pid_reader,
                    shared_policy,
                    audit_events,
                    auto_promotions,
                    fatal_error,
                    active_tracees,
                    completed_roots,
                );
            })
            .map(|_| ())
            .map_err(|error| PtraceSupervisorError::Install(format!("spawn tracer thread: {error}")))
    }

    fn tracer_loop(
        pid_reader: OwnedFd,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: Arc<Mutex<Option<String>>>,
        active_tracees: Arc<AtomicUsize>,
        completed_roots: Arc<Mutex<BTreeMap<i32, i32>>>,
    ) {
        let mut tracees = BTreeSet::new();
        let mut tracked_roots = BTreeSet::new();
        let mut pipe_closed = false;
        loop {
            if !pipe_closed {
                match read_root_pids(pid_reader.as_raw_fd()) {
                    Ok((root_pids, closed)) => {
                        pipe_closed = closed;
                        for pid in root_pids {
                            tracked_roots.insert(pid);
                            if let Err(error) = attach_root_tracee(pid, &mut tracees, &active_tracees) {
                                record_tracer_failure(
                                    pid,
                                    error.to_string(),
                                    &audit_events,
                                    &auto_promotions,
                                    &fatal_error,
                                );
                                kill_pid(pid);
                            }
                        }
                    }
                    Err(error) => {
                        pipe_closed = true;
                        record_tracer_failure(0, error.to_string(), &audit_events, &auto_promotions, &fatal_error);
                    }
                }
            }
            let snapshot = tracees.iter().copied().collect::<Vec<_>>();
            for pid in snapshot {
                if let Err(error) = drain_tracee_waits(
                    pid,
                    &mut tracees,
                    &active_tracees,
                    &shared_policy,
                    &audit_events,
                    &auto_promotions,
                    &fatal_error,
                    &mut tracked_roots,
                    &completed_roots,
                ) {
                    record_tracer_failure(pid, error.to_string(), &audit_events, &auto_promotions, &fatal_error);
                    kill_pid(pid);
                }
            }
            if pipe_closed && tracees.is_empty() {
                active_tracees.store(0, Ordering::Release);
                return;
            }
            poll_pid_pipe(pid_reader.as_raw_fd());
        }
    }

    fn read_root_pids(pid_reader: RawFd) -> Result<(Vec<i32>, bool), PtraceSupervisorError> {
        let mut pids = Vec::new();
        loop {
            let mut bytes = [0_u8; PID_BYTES];
            let result = unsafe { libc::read(pid_reader, bytes.as_mut_ptr().cast::<libc::c_void>(), bytes.len()) };
            if result == PID_BYTES as isize {
                let pid = libc::pid_t::from_ne_bytes(bytes);
                if pid <= 0 {
                    return Err(PtraceSupervisorError::Attach(format!("PID pipe returned invalid pid {pid}")));
                }
                pids.push(pid);
                continue;
            }
            if result == 0 {
                return Ok((pids, true));
            }
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::WouldBlock {
                    return Ok((pids, false));
                }
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(PtraceSupervisorError::TracerLoop(format!("read PID pipe: {error}")));
            }
            return Err(PtraceSupervisorError::TracerLoop(format!("PID pipe read was short: {result}")));
        }
    }

    fn poll_pid_pipe(pid_reader: RawFd) {
        let mut descriptor = libc::pollfd {
            fd: pid_reader,
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        };
        let result = unsafe { libc::poll(&mut descriptor, 1, TRACER_POLL_TIMEOUT_MS) };
        if result < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            thread::sleep(Duration::from_millis(TRACER_LOOP_SLEEP_MS));
        }
    }

    fn attach_root_tracee(
        pid: i32,
        tracees: &mut BTreeSet<i32>,
        active_tracees: &AtomicUsize,
    ) -> Result<(), PtraceSupervisorError> {
        seize(pid)?;
        insert_tracee(pid, tracees, active_tracees)?;
        let status = wait_for_tracee_stop(pid)?;
        if !libc::WIFSTOPPED(status) || libc::WSTOPSIG(status) != libc::SIGSTOP {
            return Err(PtraceSupervisorError::Attach(format!(
                "root tracee {pid} did not stop with SIGSTOP: status={status:#x}"
            )));
        }
        set_options(pid, ptrace_options())?;
        cont_with_signal(pid, 0)
    }

    fn seize(pid: i32) -> Result<(), PtraceSupervisorError> {
        let result = unsafe {
            libc::ptrace(
                PTRACE_SEIZE,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                ptrace_options() as usize as *mut libc::c_void,
            )
        };
        if result == 0 {
            return Ok(());
        }
        Err(PtraceSupervisorError::Attach(last_error("PTRACE_SEIZE")))
    }

    fn ptrace_options() -> libc::c_uint {
        PTRACE_O_TRACESYSGOOD
            | PTRACE_O_TRACEFORK
            | PTRACE_O_TRACEVFORK
            | PTRACE_O_TRACECLONE
            | PTRACE_O_TRACEEXEC
            | PTRACE_O_TRACESECCOMP
    }

    fn wait_for_tracee_stop(pid: i32) -> Result<i32, PtraceSupervisorError> {
        loop {
            let mut status = 0;
            let result = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::__WALL) };
            if result == pid {
                return Ok(status);
            }
            if result < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(PtraceSupervisorError::Attach(last_error("waitpid initial SIGSTOP")));
        }
    }

    fn set_options(pid: i32, options: libc::c_uint) -> Result<(), PtraceSupervisorError> {
        let result = unsafe {
            libc::ptrace(
                PTRACE_SETOPTIONS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                options as usize as *mut libc::c_void,
            )
        };
        if result == 0 {
            return Ok(());
        }
        Err(PtraceSupervisorError::Attach(last_error("PTRACE_SETOPTIONS")))
    }

    fn insert_tracee(
        pid: i32,
        tracees: &mut BTreeSet<i32>,
        active_tracees: &AtomicUsize,
    ) -> Result<(), PtraceSupervisorError> {
        if tracees.contains(&pid) {
            return Ok(());
        }
        if tracees.len() >= TRACEES_MAX {
            return Err(PtraceSupervisorError::TracerLoop(format!("tracee count exceeds bound {TRACEES_MAX}")));
        }
        tracees.insert(pid);
        active_tracees.store(tracees.len(), Ordering::Release);
        Ok(())
    }

    fn remove_tracee(pid: i32, tracees: &mut BTreeSet<i32>, active_tracees: &AtomicUsize) {
        tracees.remove(&pid);
        active_tracees.store(tracees.len(), Ordering::Release);
    }

    fn drain_tracee_waits(
        pid: i32,
        tracees: &mut BTreeSet<i32>,
        active_tracees: &AtomicUsize,
        shared_policy: &Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: &Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: &Arc<Mutex<Option<String>>>,
        tracked_roots: &mut BTreeSet<i32>,
        completed_roots: &Arc<Mutex<BTreeMap<i32, i32>>>,
    ) -> Result<(), PtraceSupervisorError> {
        loop {
            let mut status = 0;
            let result = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::__WALL | libc::WNOHANG) };
            if result == 0 {
                return Ok(());
            }
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.raw_os_error() == Some(libc::ECHILD) {
                    tracked_roots.remove(&pid);
                    remove_tracee(pid, tracees, active_tracees);
                    return Ok(());
                }
                return Err(PtraceSupervisorError::TracerLoop(format!("waitpid tracee {pid}: {error}")));
            }
            if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
                remove_tracee(pid, tracees, active_tracees);
                if tracked_roots.remove(&pid) {
                    match completed_roots.lock() {
                        Ok(mut completed) => {
                            completed.insert(pid, status);
                        }
                        Err(poisoned) => {
                            poisoned.into_inner().insert(pid, status);
                        }
                    }
                }
                return Ok(());
            }
            if !libc::WIFSTOPPED(status) {
                return Err(PtraceSupervisorError::TracerLoop(format!(
                    "tracee {pid} returned unexpected wait status {status:#x}"
                )));
            }
            handle_tracee_stop(
                pid,
                status,
                tracees,
                active_tracees,
                shared_policy,
                audit_events,
                auto_promotions,
                fatal_error,
            )?;
        }
    }

    fn handle_tracee_stop(
        pid: i32,
        status: i32,
        tracees: &mut BTreeSet<i32>,
        active_tracees: &AtomicUsize,
        shared_policy: &Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: &Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: &Arc<Mutex<Option<String>>>,
    ) -> Result<(), PtraceSupervisorError> {
        if is_seccomp_event_stop(status) {
            return handle_exec_stop(pid, status, shared_policy, audit_events, auto_promotions, fatal_error);
        }
        let event = ptrace_event(status);
        if is_descendant_event(event) {
            let child_pid = get_event_pid(pid)?;
            insert_tracee(child_pid, tracees, active_tracees)?;
            return cont_with_signal(pid, 0);
        }
        if is_initial_group_stop(status) || event == PTRACE_EVENT_STOP {
            set_options(pid, ptrace_options())?;
            return cont_with_signal(pid, 0);
        }
        if event != 0 || is_syscall_stop(status) {
            return cont_with_signal(pid, 0);
        }
        if is_signal_delivery_stop(status) {
            return cont_with_signal(pid, libc::WSTOPSIG(status));
        }
        Err(PtraceSupervisorError::TracerLoop(format!("tracee {pid} returned unclassified stop {status:#x}")))
    }

    fn handle_exec_stop(
        pid: i32,
        status: i32,
        shared_policy: &Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: &Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: &Arc<Mutex<Option<String>>>,
    ) -> Result<(), PtraceSupervisorError> {
        let stop = match exec_entry_from_stop(status, pid) {
            Ok(stop) => stop,
            Err(error) => {
                let decision = denied_decision(
                    audit_pid(pid),
                    "unknown",
                    PathBuf::new(),
                    PathBuf::new(),
                    String::new(),
                    error.to_string(),
                    true,
                );
                return apply_decision(pid, decision, audit_events, auto_promotions, fatal_error);
            }
        };
        let decision = match shared_policy.write() {
            Ok(mut policy) => classify_exec_stop(&mut policy, stop),
            Err(_) => denied_decision(
                audit_pid(pid),
                stop.syscall_name(),
                PathBuf::new(),
                PathBuf::new(),
                String::new(),
                "protected exec policy lock was poisoned".to_string(),
                true,
            ),
        };
        apply_decision(pid, decision, audit_events, auto_promotions, fatal_error)
    }

    fn apply_decision(
        pid: i32,
        mut decision: SupervisorDecision,
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: &Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: &Arc<Mutex<Option<String>>>,
    ) -> Result<(), PtraceSupervisorError> {
        let response = if decision.allowed {
            cont_with_signal(pid, 0)
        } else {
            deny_exec_entry(pid).and_then(|()| cont_with_signal(pid, 0))
        };
        if let Err(error) = response {
            decision.allowed = false;
            decision.technical_failure = true;
            decision.promotion = None;
            decision.audit_event.policy_decision = "denied".to_string();
            decision.audit_event.reason = format!("ptrace response failed: {error}");
            record_fatal(fatal_error, decision.audit_event.reason.clone());
            record_decision(decision, audit_events, auto_promotions);
            kill_pid(pid);
            return Err(error);
        }
        if decision.technical_failure {
            record_fatal(fatal_error, decision.audit_event.reason.clone());
        }
        record_decision(decision, audit_events, auto_promotions);
        Ok(())
    }

    fn record_decision(
        decision: SupervisorDecision,
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: &Arc<Mutex<Vec<OutputPromotionRecord>>>,
    ) {
        if let Some(promotion) = decision.promotion {
            match auto_promotions.lock() {
                Ok(mut promotions) => promotions.push(promotion),
                Err(poisoned) => poisoned.into_inner().push(promotion),
            }
        }
        match audit_events.lock() {
            Ok(mut events) => events.push(decision.audit_event),
            Err(poisoned) => poisoned.into_inner().push(decision.audit_event),
        }
    }

    fn record_tracer_failure(
        pid: i32,
        reason: String,
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        auto_promotions: &Arc<Mutex<Vec<OutputPromotionRecord>>>,
        fatal_error: &Arc<Mutex<Option<String>>>,
    ) {
        let decision = denied_decision(
            audit_pid(pid),
            "ptrace",
            PathBuf::new(),
            PathBuf::new(),
            String::new(),
            reason.clone(),
            true,
        );
        record_decision(decision, audit_events, auto_promotions);
        record_fatal(fatal_error, reason);
    }

    fn record_fatal(fatal_error: &Arc<Mutex<Option<String>>>, message: String) {
        let mut fatal = match fatal_error.lock() {
            Ok(fatal) => fatal,
            Err(poisoned) => poisoned.into_inner(),
        };
        if fatal.is_none() {
            *fatal = Some(message);
        }
    }

    fn classify_exec_stop(policy: &mut ProtectedExecPolicy, stop: ExecStop) -> SupervisorDecision {
        let pid = stop.pid();
        let pid_u32 = match u32::try_from(pid) {
            Ok(pid) => pid,
            Err(_) => {
                return denied_decision(
                    0,
                    stop.syscall_name(),
                    PathBuf::new(),
                    PathBuf::new(),
                    String::new(),
                    format!("tracee pid is outside u32: {pid}"),
                    true,
                );
            }
        };
        let bytes = match read_remote_cstring(pid_u32, stop.path_address()) {
            Ok(bytes) => bytes,
            Err(error) => {
                return denied_decision(
                    pid_u32,
                    stop.syscall_name(),
                    PathBuf::new(),
                    PathBuf::new(),
                    String::new(),
                    error,
                    true,
                );
            }
        };
        let tracee_path = PathBuf::from(OsString::from_vec(bytes));
        if !tracee_path.is_absolute() {
            return denied_decision(
                pid_u32,
                stop.syscall_name(),
                tracee_path,
                PathBuf::new(),
                String::new(),
                "relative exec path cannot be resolved safely".to_string(),
                false,
            );
        }
        let resolved_host_path = match resolve_tracee_exec_path(pid_u32, &tracee_path) {
            Ok(path) => path,
            Err(error) => {
                return denied_decision(
                    pid_u32,
                    stop.syscall_name(),
                    tracee_path,
                    PathBuf::new(),
                    String::new(),
                    error,
                    true,
                );
            }
        };
        let target = ExecTarget {
            tracee_path,
            resolved_host_path,
        };
        classify_target(policy, pid_u32, stop.syscall_name(), target)
    }

    fn classify_target(
        policy: &mut ProtectedExecPolicy,
        pid: u32,
        syscall_name: &'static str,
        target: ExecTarget,
    ) -> SupervisorDecision {
        let digest_hex = match blake3_file_hex(&target.resolved_host_path) {
            Ok(digest) => digest,
            Err(error) => {
                return denied_decision(
                    pid,
                    syscall_name,
                    target.tracee_path,
                    target.resolved_host_path,
                    String::new(),
                    error.to_string(),
                    true,
                );
            }
        };
        match policy.decide_exec_or_promote(&ExecRequest {
            path: target.resolved_host_path.clone(),
            digest_hex: digest_hex.clone(),
        }) {
            Ok(classification) => SupervisorDecision {
                allowed: true,
                technical_failure: false,
                promotion: classification.promotion,
                audit_event: ProtectedSeccompAuditEvent {
                    pid,
                    syscall: syscall_name.to_string(),
                    executable_path: target.resolved_host_path.clone(),
                    tracee_path: target.tracee_path,
                    resolved_host_path: target.resolved_host_path,
                    digest_hex,
                    reason: classification.decision.reason,
                    phase: PHASE_PROTECTED.to_string(),
                    inventory_entry_id: classification.decision.entry_id,
                    policy_decision: "allowed".to_string(),
                },
            },
            Err(error) => denied_decision(
                pid,
                syscall_name,
                target.tracee_path,
                target.resolved_host_path,
                digest_hex,
                error.to_string(),
                false,
            ),
        }
    }

    fn denied_decision(
        pid: u32,
        syscall_name: &'static str,
        tracee_path: PathBuf,
        resolved_host_path: PathBuf,
        digest_hex: String,
        reason: String,
        technical_failure: bool,
    ) -> SupervisorDecision {
        SupervisorDecision {
            allowed: false,
            technical_failure,
            promotion: None,
            audit_event: ProtectedSeccompAuditEvent {
                pid,
                syscall: syscall_name.to_string(),
                executable_path: resolved_host_path.clone(),
                tracee_path,
                resolved_host_path,
                digest_hex,
                reason,
                phase: PHASE_PROTECTED.to_string(),
                inventory_entry_id: None,
                policy_decision: "denied".to_string(),
            },
        }
    }

    fn read_remote_cstring(pid: u32, address: u64) -> Result<Vec<u8>, String> {
        if address == 0 {
            return Err("exec path pointer is null".to_string());
        }
        let mut buffer = vec![0_u8; MAX_REMOTE_PATH_BYTES];
        let count = read_tracee_memory(pid, address, &mut buffer)?;
        if count == 0 {
            return Err("target exec path read returned EOF".to_string());
        }
        buffer.truncate(count);
        let Some(nul_position) = buffer.iter().position(|byte| *byte == 0) else {
            return Err(format!("target exec path exceeds {MAX_REMOTE_PATH_BYTES} bytes"));
        };
        buffer.truncate(nul_position);
        if buffer.is_empty() {
            return Err("empty exec path is not supported".to_string());
        }
        Ok(buffer)
    }

    fn read_tracee_memory(pid: u32, address: u64, buffer: &mut [u8]) -> Result<usize, String> {
        let process_id = libc::pid_t::try_from(pid).map_err(|_| format!("target pid exceeds pid_t: {pid}"))?;
        let local = libc::iovec {
            iov_base: buffer.as_mut_ptr().cast::<libc::c_void>(),
            iov_len: buffer.len(),
        };
        let remote = libc::iovec {
            iov_base: usize::try_from(address).map_err(|_| format!("target address exceeds usize: {address}"))?
                as *mut libc::c_void,
            iov_len: buffer.len(),
        };
        let result = unsafe { libc::process_vm_readv(process_id, &local, 1, &remote, 1, 0) };
        if result >= 0 {
            return usize::try_from(result).map_err(|_| format!("target read size exceeds usize: {result}"));
        }
        let process_vm_error = io::Error::last_os_error();
        let memory = File::open(format!("/proc/{pid}/mem"))
            .map_err(|error| format!("read target memory: process_vm_readv={process_vm_error}; proc-mem={error}"))?;
        memory
            .read_at(buffer, address)
            .map_err(|error| format!("read target exec path: process_vm_readv={process_vm_error}; proc-mem={error}"))
    }

    fn resolve_tracee_exec_path(pid: u32, tracee_path: &Path) -> Result<PathBuf, String> {
        let proc_root = PathBuf::from(format!("/proc/{pid}/root"));
        let proc_path = join_tracee_root_path(&proc_root, tracee_path)?;
        let file = File::open(&proc_path)
            .map_err(|error| format!("open tracee exec path {}: {error}", proc_path.display()))?;
        let fd_path = PathBuf::from("/proc/self/fd").join(file.as_raw_fd().to_string());
        let resolved = std::fs::read_link(&fd_path)
            .map_err(|error| format!("resolve tracee exec fd {}: {error}", fd_path.display()))?;
        if !resolved.is_absolute() {
            return Err(format!("resolved tracee exec path is not absolute: {}", resolved.display()));
        }
        Ok(resolved)
    }

    fn join_tracee_root_path(tracee_root: &Path, tracee_path: &Path) -> Result<PathBuf, String> {
        if !tracee_root.is_absolute() {
            return Err(format!("tracee root is not absolute: {}", tracee_root.display()));
        }
        if !tracee_path.is_absolute() {
            return Err("relative exec path cannot be resolved safely".to_string());
        }
        let mut output = tracee_root.to_path_buf();
        for component in tracee_path.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(part) => output.push(part),
                Component::ParentDir => {
                    return Err(format!(
                        "tracee exec path contains unsupported parent component: {}",
                        tracee_path.display()
                    ));
                }
                Component::Prefix(_) => {
                    return Err(format!("tracee exec path contains unsupported prefix: {}", tracee_path.display()));
                }
            }
        }
        Ok(output)
    }

    fn get_entry_regs(pid: i32) -> Result<EntryRegs, PtraceSupervisorError> {
        let mut regs: libc::user_regs_struct = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::ptrace(
                PTRACE_GETREGS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &mut regs as *mut libc::user_regs_struct as *mut libc::c_void,
            )
        };
        if result != 0 {
            return Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_GETREGS")));
        }
        Ok(EntryRegs {
            syscall_nr: regs.orig_rax,
            arg0: regs.rdi,
            arg1: regs.rsi,
        })
    }

    fn deny_exec_entry(pid: i32) -> Result<(), PtraceSupervisorError> {
        let mut regs: libc::user_regs_struct = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::ptrace(
                PTRACE_GETREGS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &mut regs as *mut libc::user_regs_struct as *mut libc::c_void,
            )
        };
        if result != 0 {
            return Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_GETREGS deny")));
        }
        regs.rax = DENIED_RETURN;
        regs.orig_rax = DENIED_SYSCALL_MARKER;
        let result = unsafe {
            libc::ptrace(
                PTRACE_SETREGS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &regs as *const libc::user_regs_struct as *const libc::c_void,
            )
        };
        if result == 0 {
            return Ok(());
        }
        Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_SETREGS deny")))
    }

    fn cont_with_signal(pid: i32, signal: libc::c_int) -> Result<(), PtraceSupervisorError> {
        let signal_data = usize::try_from(signal).unwrap_or(0) as *mut libc::c_void;
        let result =
            unsafe { libc::ptrace(PTRACE_CONT, pid as libc::pid_t, std::ptr::null_mut::<libc::c_void>(), signal_data) };
        if result == 0 {
            return Ok(());
        }
        Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_CONT")))
    }

    fn get_event_pid(pid: i32) -> Result<i32, PtraceSupervisorError> {
        let mut child_pid: libc::c_ulong = 0;
        let result = unsafe {
            libc::ptrace(
                PTRACE_GETEVENTMSG,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &mut child_pid as *mut libc::c_ulong as *mut libc::c_void,
            )
        };
        if result != 0 {
            return Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_GETEVENTMSG")));
        }
        i32::try_from(child_pid)
            .map_err(|_| PtraceSupervisorError::TracerLoop(format!("descendant pid exceeds i32: {child_pid}")))
    }

    fn audit_pid(pid: i32) -> u32 {
        u32::try_from(pid).unwrap_or(0)
    }

    fn kill_pid(pid: i32) {
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
        }
    }

    fn exec_entry_from_stop(wait_status: i32, pid: i32) -> Result<ExecStop, PtraceSupervisorError> {
        if !is_seccomp_event_stop(wait_status) {
            return Err(PtraceSupervisorError::TracerLoop(format!("tracee {pid} is not at a seccomp event stop")));
        }
        let regs = get_entry_regs(pid)?;
        if regs.syscall_nr == EXECVE_NR && regs.arg0 != 0 {
            return Ok(ExecStop::Execve {
                pid,
                path_address: regs.arg0,
            });
        }
        if regs.syscall_nr == EXECVEAT_NR {
            let dirfd = i64::from_ne_bytes(regs.arg0.to_ne_bytes());
            if dirfd == AT_FDCWD && regs.arg1 != 0 {
                return Ok(ExecStop::Execveat {
                    pid,
                    path_address: regs.arg1,
                });
            }
            return Err(PtraceSupervisorError::TracerLoop(format!(
                "unsupported execveat dirfd {dirfd}; only AT_FDCWD is supported"
            )));
        }
        Err(PtraceSupervisorError::TracerLoop(format!(
            "seccomp event stop for unsupported syscall {}",
            regs.syscall_nr
        )))
    }

    fn is_seccomp_event_stop(wait_status: i32) -> bool {
        ptrace_event(wait_status) == PTRACE_EVENT_SECCOMP
            && wait_status_low_byte(wait_status) == STOP_LOW_BYTE_MARKER
            && wait_status_stop_signal(wait_status) == SIGTRAP_BYTE
    }

    fn is_syscall_stop(wait_status: i32) -> bool {
        wait_status_low_byte(wait_status) == STOP_LOW_BYTE_MARKER
            && wait_status_stop_signal(wait_status) == SYSCALL_STOP_SIGTRAP_BYTE
    }

    fn is_signal_delivery_stop(wait_status: i32) -> bool {
        libc::WIFSTOPPED(wait_status)
            && ptrace_event(wait_status) == 0
            && !is_syscall_stop(wait_status)
            && libc::WSTOPSIG(wait_status) != libc::SIGSTOP
    }

    fn is_initial_group_stop(wait_status: i32) -> bool {
        libc::WIFSTOPPED(wait_status) && ptrace_event(wait_status) == 0 && libc::WSTOPSIG(wait_status) == libc::SIGSTOP
    }

    fn is_descendant_event(event: u32) -> bool {
        matches!(event, PTRACE_EVENT_FORK | PTRACE_EVENT_VFORK | PTRACE_EVENT_CLONE)
    }

    fn ptrace_event(wait_status: i32) -> u32 {
        (wait_status as u32 >> 16) & u32::from(u8::MAX)
    }

    fn wait_status_low_byte(wait_status: i32) -> u32 {
        wait_status as u32 & u32::from(u8::MAX)
    }

    fn wait_status_stop_signal(wait_status: i32) -> u32 {
        (wait_status as u32 >> 8) & u32::from(u8::MAX)
    }

    fn ptracer_io_error(error: PtraceSupervisorError) -> io::Error {
        io::Error::other(error.to_string())
    }

    fn last_error(context: &str) -> String {
        format!("{context}: {}", io::Error::last_os_error())
    }

    #[cfg(test)]
    mod tests {
        use std::os::unix::fs::PermissionsExt;

        use super::*;
        use crate::protected_exec::PlannedExecutable;

        const TEST_ARCH_X86_64: u32 = 0xC000_003E;
        const TEST_ARCH_ARM64: u32 = 0xC000_00B7;
        const CONCURRENT_WORKERS: usize = 4;
        const CONCURRENT_EXECS_PER_WORKER: usize = 128;
        const DESCENDANT_EXEC_EVENT_COUNT: usize = 2;
        const BLAKE3_HEX_LENGTH: usize = 64;
        const TEST_SHELL_PATH: &str = "/run/current-system/sw/bin/sh";
        const OUTPUT_SENTINEL: &str = "ptrace-output-ok";

        fn executable_policy(path: &Path, digest_hex: String) -> ProtectedExecPolicy {
            ProtectedExecPolicy::from_action_plan(&["ptrace-test".to_string()], &[PlannedExecutable {
                authorization_id: "ptrace-test-executable".to_string(),
                source_stage_id: "ptrace-test".to_string(),
                path: path.to_path_buf(),
                digest_hex,
            }])
            .unwrap()
        }

        #[test]
        fn unsupported_architecture_is_rejected_before_filtering() {
            if !cfg!(target_arch = "x86_64") {
                return;
            }
            let error = require_supported_audit_arch(None).unwrap_err();
            assert_eq!(
                error.to_string(),
                "ptrace supervisor unsupported: unsupported Linux audit architecture for exec supervisor"
            );
        }

        #[test]
        fn trace_filter_traces_only_exec_syscalls_on_supported_arch() {
            let filter = exec_trace_filter(TEST_ARCH_X86_64);
            assert_eq!(filter[0].k, SECCOMP_DATA_ARCH_OFFSET);
            assert_eq!(filter[2].k, libc::SECCOMP_RET_KILL_PROCESS);
            assert_eq!(filter[FILTER_EXECVE_INDEX].k, EXECVE_NR as u32);
            assert_eq!(filter[FILTER_EXECVE_INDEX].jt, 1);
            assert_eq!(filter[FILTER_EXECVE_INDEX].jf, 0);
            assert_eq!(filter[FILTER_EXECVEAT_INDEX].k, EXECVEAT_NR as u32);
            assert_eq!(filter[FILTER_EXECVEAT_INDEX].jt, 0);
            assert_eq!(filter[FILTER_EXECVEAT_INDEX].jf, 1);
            assert_eq!(filter[FILTER_TRACE_INDEX].k, SECCOMP_RET_TRACE);
            assert_eq!(filter[FILTER_ALLOW_INDEX].k, SECCOMP_RET_ALLOW);
            let arm_filter = exec_trace_filter(TEST_ARCH_ARM64);
            assert_eq!(arm_filter[1].k, TEST_ARCH_ARM64);
            assert_eq!(arm_filter[FILTER_TRACE_INDEX].k, SECCOMP_RET_TRACE);
        }

        #[test]
        fn seccomp_event_stops_are_recognized_and_signal_stops_are_distinct() {
            let event_status = (PTRACE_EVENT_SECCOMP << 16) | (SIGTRAP_BYTE << 8) | STOP_LOW_BYTE_MARKER;
            assert!(is_seccomp_event_stop(event_status as i32));
            assert!(!is_syscall_stop(event_status as i32));
            assert!(!is_signal_delivery_stop(event_status as i32));
            let syscall_status = (SYSCALL_STOP_SIGTRAP_BYTE << 8) | STOP_LOW_BYTE_MARKER;
            assert!(is_syscall_stop(syscall_status as i32));
            assert!(!is_seccomp_event_stop(syscall_status as i32));
            let signal_status = ((libc::SIGTERM as u32) << 8) | STOP_LOW_BYTE_MARKER;
            assert!(is_signal_delivery_stop(signal_status as i32));
            assert!(!is_initial_group_stop(signal_status as i32));
            let group_status = ((libc::SIGSTOP as u32) << 8) | STOP_LOW_BYTE_MARKER;
            assert!(is_initial_group_stop(group_status as i32));
        }

        #[test]
        fn ptrace_supervisor_allows_declared_exec_and_records_exact_bytes() {
            let executable = std::fs::canonicalize(TEST_SHELL_PATH).unwrap();
            let digest = blake3_file_hex(&executable).unwrap();
            let supervisor = install_exec_supervisor(executable_policy(&executable, digest.clone())).unwrap();
            let mut command = Command::new(TEST_SHELL_PATH);
            command.args(["-c", ":"]);
            let status = supervisor.status(&mut command).unwrap();
            assert!(status.success());
            supervisor.wait_for_audit_quiescence().unwrap();
            let events = supervisor.audit_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "allowed");
            assert_eq!(events[0].resolved_host_path, executable);
            assert_eq!(events[0].digest_hex, digest);
        }

        #[test]
        fn ptrace_supervisor_denies_digest_mismatch_before_exec() {
            let executable = std::fs::canonicalize(TEST_SHELL_PATH).unwrap();
            let supervisor =
                install_exec_supervisor(executable_policy(&executable, "0".repeat(BLAKE3_HEX_LENGTH))).unwrap();
            let mut command = Command::new(TEST_SHELL_PATH);
            command.args(["-c", ":"]);
            let error = supervisor.status(&mut command).unwrap_err();
            assert_eq!(error.raw_os_error(), Some(libc::EACCES));
            supervisor.wait_for_audit_quiescence().unwrap();
            let events = supervisor.audit_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert!(events[0].reason.contains("digest"));
        }

        #[test]
        fn ptrace_supervisor_auto_attaches_descendant_and_forwards_sigchld() {
            let executable = std::fs::canonicalize(TEST_SHELL_PATH).unwrap();
            let digest = blake3_file_hex(&executable).unwrap();
            let supervisor = install_exec_supervisor(executable_policy(&executable, digest)).unwrap();
            let mut command = Command::new(TEST_SHELL_PATH);
            command.args(["-c", "/run/current-system/sw/bin/sh -c ':' & wait"]);
            let status = supervisor.status(&mut command).unwrap();
            assert!(status.success());
            supervisor.wait_for_audit_quiescence().unwrap();
            let events = supervisor.audit_events();
            assert_eq!(events.len(), DESCENDANT_EXEC_EVENT_COUNT);
            assert!(events.iter().all(|event| event.policy_decision == "allowed"));
            assert!(events.iter().all(|event| event.resolved_host_path == executable));
        }

        #[test]
        fn ptrace_supervisor_output_captures_bytes_without_command_wait() {
            let executable = std::fs::canonicalize(TEST_SHELL_PATH).unwrap();
            let digest = blake3_file_hex(&executable).unwrap();
            let supervisor = install_exec_supervisor(executable_policy(&executable, digest)).unwrap();
            let mut command = Command::new(TEST_SHELL_PATH);
            command.args(["-c", "printf ptrace-output-ok"]);
            let output = supervisor.output(&mut command).unwrap();
            assert!(output.status.success());
            assert_eq!(output.stdout, OUTPUT_SENTINEL.as_bytes());
            assert!(output.stderr.is_empty());
            supervisor.wait_for_audit_quiescence().unwrap();
            assert_eq!(supervisor.audit_events().len(), 1);
        }

        #[test]
        fn ptrace_supervisor_fails_closed_when_exec_bytes_disappear() {
            let directory = tempfile::tempdir().unwrap();
            let executable = directory.path().join("vanishing-exec");
            std::fs::write(&executable, "#!/run/current-system/sw/bin/sh\nexit 0\n").unwrap();
            let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&executable, permissions).unwrap();
            let executable = std::fs::canonicalize(executable).unwrap();
            let digest = blake3_file_hex(&executable).unwrap();
            let supervisor = install_exec_supervisor(executable_policy(&executable, digest)).unwrap();
            std::fs::remove_file(&executable).unwrap();
            let mut command = Command::new(&executable);
            let error = supervisor.status(&mut command).unwrap_err();
            assert_eq!(error.raw_os_error(), Some(libc::EACCES));
            let fatal = supervisor.wait_for_audit_quiescence().unwrap_err();
            assert!(fatal.to_string().contains("open tracee exec path"));
            let events = supervisor.audit_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert!(events[0].reason.contains("open tracee exec path"));
        }

        #[test]
        fn ptrace_register_and_memory_failures_are_explicit() {
            let register_error = get_entry_regs(i32::MAX).unwrap_err();
            let mut buffer = [0_u8; 1];
            let memory_error = read_tracee_memory(u32::MAX, 1, &mut buffer).unwrap_err();
            assert!(register_error.to_string().contains("PTRACE_GETREGS"));
            assert_eq!(memory_error, format!("target pid exceeds pid_t: {}", u32::MAX));
        }

        #[test]
        fn ptrace_supervisor_handles_concurrent_declared_execs() {
            let executable = std::fs::canonicalize(TEST_SHELL_PATH).unwrap();
            let digest = blake3_file_hex(&executable).unwrap();
            let supervisor = install_exec_supervisor(executable_policy(&executable, digest)).unwrap();
            thread::scope(|scope| {
                for _ in 0..CONCURRENT_WORKERS {
                    let supervisor = &supervisor;
                    scope.spawn(move || {
                        for _ in 0..CONCURRENT_EXECS_PER_WORKER {
                            let mut command = Command::new(TEST_SHELL_PATH);
                            command.args(["-c", ":"]);
                            assert!(supervisor.status(&mut command).unwrap().success());
                        }
                    });
                }
            });
            supervisor.wait_for_audit_quiescence().unwrap();
            assert_eq!(supervisor.audit_events().len(), CONCURRENT_WORKERS.saturating_mul(CONCURRENT_EXECS_PER_WORKER));
        }
    }
}

pub use linux::*;
