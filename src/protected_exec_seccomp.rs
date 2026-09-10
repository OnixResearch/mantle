// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in the store-capability-migration change evidence and scheduled for the
// standalone hardening pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::usize_in_public_api
)]

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::BTreeSet;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io;
    use std::os::fd::AsRawFd;
    use std::os::fd::RawFd;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::FileExt;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::RwLock;
    use std::thread;

    use crate::protected_exec::ExecRequest;
    use crate::protected_exec::OutputPromotionRecord;
    use crate::protected_exec::PHASE_PROTECTED;
    use crate::protected_exec::PromotedExecutable;
    use crate::protected_exec::ProtectedExecError;
    use crate::protected_exec::ProtectedExecPolicy;
    use crate::protected_exec::ProtectedSeccompAuditEvent;
    use crate::protected_exec::blake3_file_hex;

    const AUDIT_ARCH_X86_64: u32 = 0xC000_003E;
    const AUDIT_ARCH_AARCH64: u32 = 0xC000_00B7;
    const SECCOMP_IOCTL_NOTIF_RECV: libc::c_ulong = 0xC050_2100;
    const SECCOMP_IOCTL_NOTIF_SEND: libc::c_ulong = 0xC018_2101;
    const SECCOMP_IOCTL_NOTIF_ID_VALID: libc::c_ulong = 0x4008_2102;
    const SECCOMP_DATA_NR_OFFSET: u32 = 0;
    const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
    const MAX_REMOTE_PATH_BYTES: usize = 4096;
    const FILTER_INSTRUCTION_COUNT: usize = 8;
    const EXECVE_PATH_ARG_INDEX: usize = 0;
    const EXECVEAT_DIRFD_ARG_INDEX: usize = 0;
    const EXECVEAT_PATH_ARG_INDEX: usize = 1;
    const PROC_FD_PATH_PREFIX: &str = "/proc/self/fd";
    const DIAGNOSTIC_EXEC_PATH_COUNT_MAX: usize = 256;
    const DIAGNOSTIC_EXEC_EVENT_COUNT_MAX: usize = 131_072;
    const PROTECTED_EXEC_EVENT_COUNT_MAX: usize = 131_072;
    const ADOPTED_DESCENDANT_REAP_POLL_COUNT_MAX: u32 = 3_000;
    const ADOPTED_DESCENDANT_REAP_POLL_INTERVAL_MS: u64 = 10;
    const AUDIT_QUIESCENCE_POLL_COUNT_MAX: u32 = 3_000;
    const AUDIT_QUIESCENCE_STABLE_POLL_COUNT: u32 = 100;
    const AUDIT_QUIESCENCE_POLL_INTERVAL_MS: u64 = 10;
    const _: () = assert!(ADOPTED_DESCENDANT_REAP_POLL_COUNT_MAX > 0);
    const _: () = assert!(AUDIT_QUIESCENCE_STABLE_POLL_COUNT > 0);
    const PHASE_DIAGNOSTIC: &str = "diagnostic";

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum ProtectedSeccompError {
        Unsupported(String),
        Install(String),
        Supervisor(String),
    }

    impl std::fmt::Display for ProtectedSeccompError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unsupported(message) => write!(f, "protected exec supervisor unsupported: {message}"),
                Self::Install(message) => write!(f, "installing protected exec supervisor failed: {message}"),
                Self::Supervisor(message) => write!(f, "protected exec supervisor failed: {message}"),
            }
        }
    }

    impl std::error::Error for ProtectedSeccompError {}

    #[derive(Debug)]
    pub struct DiagnosticExecObserver {
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        listener_fd: RawFd,
    }

    impl DiagnosticExecObserver {
        pub fn audit_events(&self) -> Vec<ProtectedSeccompAuditEvent> {
            match self.audit_events.lock() {
                Ok(events) => events.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }

        pub fn listener_fd(&self) -> RawFd {
            self.listener_fd
        }

        pub fn wait_for_audit_quiescence(&self) -> Result<usize, ProtectedSeccompError> {
            wait_for_audit_quiescence(&self.audit_events)
        }
    }

    #[derive(Debug)]
    pub struct ProtectedSeccompSupervisor {
        // Lock order: shared_policy -> audit_events. Never acquire these locks in reverse order.
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        listener_fd: RawFd,
    }

    impl ProtectedSeccompSupervisor {
        pub fn audit_events(&self) -> Vec<ProtectedSeccompAuditEvent> {
            match self.audit_events.lock() {
                Ok(events) => events.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }

        pub fn listener_fd(&self) -> RawFd {
            self.listener_fd
        }

        pub fn wait_for_audit_quiescence(&self) -> Result<usize, ProtectedSeccompError> {
            wait_for_audit_quiescence(&self.audit_events)
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
    }

    pub fn reap_adopted_exec_descendants() -> Result<u32, ProtectedSeccompError> {
        let mut reaped_count = 0_u32;
        for _ in 0..ADOPTED_DESCENDANT_REAP_POLL_COUNT_MAX {
            let mut status: libc::c_int = 0;
            let wait_result = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
            if wait_result > 0 {
                if !libc::WIFEXITED(status) || libc::WEXITSTATUS(status) != 0 {
                    return Err(ProtectedSeccompError::Supervisor(format!(
                        "adopted StageX descendant {wait_result} failed with wait status {status}"
                    )));
                }
                reaped_count = reaped_count.checked_add(1).ok_or_else(|| {
                    ProtectedSeccompError::Supervisor("adopted descendant reap count overflow".to_string())
                })?;
                continue;
            }
            if wait_result == 0 {
                thread::sleep(std::time::Duration::from_millis(ADOPTED_DESCENDANT_REAP_POLL_INTERVAL_MS));
                continue;
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            if error.raw_os_error() == Some(libc::ECHILD) {
                assert!(reaped_count < u32::MAX);
                return Ok(reaped_count);
            }
            return Err(ProtectedSeccompError::Supervisor(format!("reaping adopted StageX descendant: {error}")));
        }
        Err(ProtectedSeccompError::Supervisor(format!(
            "adopted StageX descendants did not exit within {} ms",
            u64::from(ADOPTED_DESCENDANT_REAP_POLL_COUNT_MAX).saturating_mul(ADOPTED_DESCENDANT_REAP_POLL_INTERVAL_MS)
        )))
    }

    fn wait_for_audit_quiescence(
        audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) -> Result<usize, ProtectedSeccompError> {
        let mut previous_count = usize::MAX;
        let mut stable_poll_count = 0_u32;
        for _ in 0..AUDIT_QUIESCENCE_POLL_COUNT_MAX {
            let count = match audit_events.lock() {
                Ok(events) => events.len(),
                Err(poisoned) => poisoned.into_inner().len(),
            };
            if count == previous_count {
                stable_poll_count = stable_poll_count.saturating_add(1);
            } else {
                previous_count = count;
                stable_poll_count = 0;
            }
            if stable_poll_count >= AUDIT_QUIESCENCE_STABLE_POLL_COUNT {
                assert!(count <= PROTECTED_EXEC_EVENT_COUNT_MAX);
                return Ok(count);
            }
            thread::sleep(std::time::Duration::from_millis(AUDIT_QUIESCENCE_POLL_INTERVAL_MS));
        }
        Err(ProtectedSeccompError::Supervisor(format!(
            "StageX exec audit did not become quiescent within {} ms",
            u64::from(AUDIT_QUIESCENCE_POLL_COUNT_MAX).saturating_mul(AUDIT_QUIESCENCE_POLL_INTERVAL_MS)
        )))
    }

    pub fn install_current_thread_diagnostic_exec_observer(
        allowed_paths: BTreeSet<PathBuf>,
    ) -> Result<DiagnosticExecObserver, ProtectedSeccompError> {
        validate_diagnostic_paths(&allowed_paths)?;
        verify_notification_sizes()?;
        set_child_subreaper()?;
        set_no_new_privileges()?;
        let listener_fd = install_exec_filter()?;
        let audit_events = Arc::new(Mutex::new(Vec::new()));
        spawn_diagnostic_observer_thread(listener_fd, allowed_paths, audit_events.clone())?;
        Ok(DiagnosticExecObserver {
            audit_events,
            listener_fd,
        })
    }

    fn validate_diagnostic_paths(allowed_paths: &BTreeSet<PathBuf>) -> Result<(), ProtectedSeccompError> {
        if allowed_paths.is_empty() || allowed_paths.len() > DIAGNOSTIC_EXEC_PATH_COUNT_MAX {
            return Err(ProtectedSeccompError::Install(format!(
                "diagnostic exec path count must be in 1..={DIAGNOSTIC_EXEC_PATH_COUNT_MAX}"
            )));
        }
        if let Some(path) = allowed_paths.iter().find(|path| !path.is_absolute()) {
            return Err(ProtectedSeccompError::Install(format!(
                "diagnostic exec path is not absolute: {}",
                path.display()
            )));
        }
        assert!(!allowed_paths.is_empty());
        assert!(allowed_paths.iter().all(|path| path.is_absolute()));
        Ok(())
    }

    pub fn install_current_thread_exec_supervisor(
        policy: ProtectedExecPolicy,
    ) -> Result<ProtectedSeccompSupervisor, ProtectedSeccompError> {
        verify_notification_sizes()?;
        set_child_subreaper()?;
        set_no_new_privileges()?;
        let listener_fd = install_exec_filter()?;
        let audit_events = Arc::new(Mutex::new(Vec::new()));
        let shared_policy = Arc::new(RwLock::new(policy));
        spawn_supervisor_thread(listener_fd, shared_policy.clone(), audit_events.clone())?;
        Ok(ProtectedSeccompSupervisor {
            audit_events,
            shared_policy,
            listener_fd,
        })
    }

    fn verify_notification_sizes() -> Result<(), ProtectedSeccompError> {
        let mut notification_sizes_bytes: libc::seccomp_notif_sizes = unsafe { std::mem::zeroed() };
        let rc = unsafe {
            libc::syscall(libc::SYS_seccomp, libc::SECCOMP_GET_NOTIF_SIZES, 0, &mut notification_sizes_bytes)
        };
        if rc == 0 {
            return Ok(());
        }
        Err(ProtectedSeccompError::Unsupported(last_os_error("SECCOMP_GET_NOTIF_SIZES")))
    }

    fn set_child_subreaper() -> Result<(), ProtectedSeccompError> {
        let rc = unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) };
        if rc == 0 {
            return Ok(());
        }
        Err(ProtectedSeccompError::Install(last_os_error("PR_SET_CHILD_SUBREAPER")))
    }

    fn set_no_new_privileges() -> Result<(), ProtectedSeccompError> {
        let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if rc == 0 {
            return Ok(());
        }
        Err(ProtectedSeccompError::Install(last_os_error("PR_SET_NO_NEW_PRIVS")))
    }

    fn install_exec_filter() -> Result<RawFd, ProtectedSeccompError> {
        let arch = require_supported_audit_arch(current_audit_arch())?;
        let mut filter = exec_filter(arch)?;
        let filter_len = u16::try_from(filter.len())
            .map_err(|_| ProtectedSeccompError::Install("seccomp filter length exceeds u16".to_string()))?;
        let mut program = libc::sock_fprog {
            len: filter_len,
            filter: filter.as_mut_ptr(),
        };
        assert_eq!(filter.len(), FILTER_INSTRUCTION_COUNT);
        assert!(!program.filter.is_null());
        let fd = unsafe {
            libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_SET_MODE_FILTER,
                libc::SECCOMP_FILTER_FLAG_NEW_LISTENER,
                &mut program,
            )
        };
        if fd < 0 {
            return Err(ProtectedSeccompError::Install(last_os_error("SECCOMP_SET_MODE_FILTER")));
        }
        RawFd::try_from(fd).map_err(|_| ProtectedSeccompError::Install("seccomp listener fd exceeds i32".to_string()))
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

    fn exec_filter(arch: u32) -> Result<[libc::sock_filter; FILTER_INSTRUCTION_COUNT], ProtectedSeccompError> {
        let execve_nr = filter_syscall_number(libc::SYS_execve, "execve")?;
        let execveat_nr = filter_syscall_number(libc::SYS_execveat, "execveat")?;
        assert_ne!(arch, 0);
        assert_ne!(execve_nr, execveat_nr);
        Ok([
            bpf_stmt(BpfStatement {
                code: libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                operand: SECCOMP_DATA_ARCH_OFFSET,
            })?,
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: arch,
                jump_true: 1,
                jump_false: 0,
            })?,
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: libc::SECCOMP_RET_KILL_PROCESS,
            })?,
            bpf_stmt(BpfStatement {
                code: libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                operand: SECCOMP_DATA_NR_OFFSET,
            })?,
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: execve_nr,
                jump_true: 1,
                jump_false: 0,
            })?,
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: execveat_nr,
                jump_true: 0,
                jump_false: 1,
            })?,
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: libc::SECCOMP_RET_USER_NOTIF,
            })?,
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: libc::SECCOMP_RET_ALLOW,
            })?,
        ])
    }

    fn filter_syscall_number(syscall: libc::c_long, name: &'static str) -> Result<u32, ProtectedSeccompError> {
        u32::try_from(syscall)
            .map_err(|_| ProtectedSeccompError::Unsupported(format!("{name} syscall number is outside u32")))
    }

    fn bpf_stmt(statement: BpfStatement) -> Result<libc::sock_filter, ProtectedSeccompError> {
        let code = u16::try_from(statement.code)
            .map_err(|_| ProtectedSeccompError::Install("seccomp BPF statement code exceeds u16".to_string()))?;
        Ok(libc::sock_filter {
            code,
            jt: 0,
            jf: 0,
            k: statement.operand,
        })
    }

    fn bpf_jump(jump: BpfJump) -> Result<libc::sock_filter, ProtectedSeccompError> {
        let code = u16::try_from(jump.code)
            .map_err(|_| ProtectedSeccompError::Install("seccomp BPF jump code exceeds u16".to_string()))?;
        Ok(libc::sock_filter {
            code,
            jt: jump.jump_true,
            jf: jump.jump_false,
            k: jump.operand,
        })
    }

    fn require_supported_audit_arch(arch: Option<u32>) -> Result<u32, ProtectedSeccompError> {
        arch.ok_or_else(|| {
            ProtectedSeccompError::Unsupported("unsupported Linux audit architecture for exec supervisor".to_string())
        })
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

    fn spawn_diagnostic_observer_thread(
        listener_fd: RawFd,
        allowed_paths: BTreeSet<PathBuf>,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) -> Result<(), ProtectedSeccompError> {
        thread::Builder::new()
            .name("mantle-diagnostic-exec-observer".to_string())
            .spawn(move || diagnostic_observer_loop(listener_fd, allowed_paths, audit_events))
            .map(|_| ())
            .map_err(|err| ProtectedSeccompError::Supervisor(format!("spawning diagnostic observer thread: {err}")))
    }

    fn diagnostic_observer_loop(
        listener_fd: RawFd,
        allowed_paths: BTreeSet<PathBuf>,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) {
        while listener_is_open(listener_fd) {
            let mut notif: libc::seccomp_notif = unsafe { std::mem::zeroed() };
            #[allow(clippy::unnecessary_cast)]
            let recv_rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_RECV as libc::Ioctl, &mut notif) };
            if recv_rc != 0 {
                let err = io::Error::last_os_error();
                if matches!(err.raw_os_error(), Some(libc::EINTR) | Some(libc::ENOENT)) {
                    continue;
                }
                return;
            }
            if event_limit_reached(&audit_events, DIAGNOSTIC_EXEC_EVENT_COUNT_MAX) {
                // Deny-response send failure is best-effort: the tracee is already
                // blocked and the supervisor loop must keep draining notifications.
                #[allow(tigerstyle::ignored_result)]
                let _ = send_response(listener_fd, notif.id, false);
                continue;
            }
            let decision = classify_diagnostic_notification(listener_fd, &allowed_paths, &notif);
            match audit_events.lock() {
                Ok(mut events) => events.push(decision.audit_event),
                Err(poisoned) => poisoned.into_inner().push(decision.audit_event),
            }
            if send_response(listener_fd, notif.id, decision.allowed).is_err() {
                continue;
            }
        }
    }

    fn event_limit_reached(audit_events: &Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>, count_max: usize) -> bool {
        let count = match audit_events.lock() {
            Ok(events) => events.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        };
        assert!(count <= count_max);
        assert!(count_max > 0);
        count >= count_max
    }

    fn spawn_supervisor_thread(
        listener_fd: RawFd,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) -> Result<(), ProtectedSeccompError> {
        thread::Builder::new()
            .name("crunch-protected-exec-supervisor".to_string())
            .spawn(move || supervisor_loop(listener_fd, shared_policy, audit_events))
            .map(|_| ())
            .map_err(|err| ProtectedSeccompError::Supervisor(format!("spawning supervisor thread: {err}")))
    }

    fn supervisor_loop(
        listener_fd: RawFd,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) {
        while listener_is_open(listener_fd) {
            let mut notif: libc::seccomp_notif = unsafe { std::mem::zeroed() };
            #[allow(clippy::unnecessary_cast)]
            let recv_rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_RECV as libc::Ioctl, &mut notif) };
            if recv_rc != 0 {
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                if err.raw_os_error() == Some(libc::ENOENT) {
                    continue;
                }
                return;
            }
            if event_limit_reached(&audit_events, PROTECTED_EXEC_EVENT_COUNT_MAX) {
                // Deny-response send failure is best-effort: the tracee is already
                // blocked and the supervisor loop must keep draining notifications.
                #[allow(tigerstyle::ignored_result)]
                let _ = send_response(listener_fd, notif.id, false);
                continue;
            }
            let decision = match shared_policy.read() {
                Ok(policy) => classify_notification(listener_fd, &policy, &notif),
                Err(_) => denied_event(DeniedEventInput {
                    pid: notif.pid,
                    syscall_name: syscall_name(notif.data.nr),
                    tracee_path: PathBuf::new(),
                    resolved_host_path: PathBuf::new(),
                    digest_hex: String::new(),
                    reason: "protected exec policy lock is poisoned".to_string(),
                    inventory_entry_id: None,
                }),
            };
            match audit_events.lock() {
                Ok(mut events) => events.push(decision.audit_event),
                Err(poisoned) => poisoned.into_inner().push(decision.audit_event),
            }
            if send_response(listener_fd, notif.id, decision.allowed).is_err() {
                continue;
            }
        }
    }

    fn listener_is_open(listener_fd: RawFd) -> bool {
        let rc = unsafe { libc::fcntl(listener_fd, libc::F_GETFD) };
        if rc >= 0 {
            return true;
        }
        io::Error::last_os_error().raw_os_error() != Some(libc::EBADF)
    }

    struct SupervisorDecision {
        allowed: bool,
        audit_event: ProtectedSeccompAuditEvent,
    }

    struct ExecTarget {
        tracee_path: PathBuf,
        resolved_host_path: PathBuf,
    }

    fn classify_diagnostic_notification(
        listener_fd: RawFd,
        allowed_paths: &BTreeSet<PathBuf>,
        notif: &libc::seccomp_notif,
    ) -> SupervisorDecision {
        let syscall_name = syscall_name(notif.data.nr);
        match exec_target(listener_fd, notif) {
            Ok(target) => classify_diagnostic_path(allowed_paths, notif.pid, syscall_name, target),
            Err((path, reason)) => denied_event(DeniedEventInput {
                pid: notif.pid,
                syscall_name,
                tracee_path: path,
                resolved_host_path: PathBuf::new(),
                digest_hex: String::new(),
                reason: format!("{reason}; {}", diagnostic_tracee_context(notif.pid)),
                inventory_entry_id: None,
            }),
        }
    }

    fn diagnostic_tracee_context(pid: u32) -> String {
        match std::fs::read_link(format!("/proc/{pid}/exe")) {
            Ok(path) => format!("tracee-image={}", path.display()),
            Err(error) => format!("tracee-image-unavailable={error}"),
        }
    }

    fn classify_diagnostic_path(
        allowed_paths: &BTreeSet<PathBuf>,
        pid: u32,
        syscall_name: &'static str,
        target: ExecTarget,
    ) -> SupervisorDecision {
        if !diagnostic_path_allowed(allowed_paths, &target) {
            return denied_event(DeniedEventInput {
                pid,
                syscall_name,
                tracee_path: target.tracee_path,
                resolved_host_path: target.resolved_host_path,
                digest_hex: String::new(),
                reason: "path is outside the exact diagnostic exec inventory".to_string(),
                inventory_entry_id: None,
            });
        }
        let digest_hex = match blake3_file_hex(&target.resolved_host_path) {
            Ok(digest) => digest,
            Err(err) => {
                return denied_event(DeniedEventInput {
                    pid,
                    syscall_name,
                    tracee_path: target.tracee_path,
                    resolved_host_path: target.resolved_host_path,
                    digest_hex: String::new(),
                    reason: err.to_string(),
                    inventory_entry_id: None,
                });
            }
        };
        SupervisorDecision {
            allowed: true,
            audit_event: ProtectedSeccompAuditEvent {
                pid,
                syscall: syscall_name.to_string(),
                executable_path: target.resolved_host_path.clone(),
                tracee_path: target.tracee_path,
                resolved_host_path: target.resolved_host_path,
                digest_hex,
                reason: "diagnostic path observation only; this event grants no protected authority".to_string(),
                phase: PHASE_DIAGNOSTIC.to_string(),
                inventory_entry_id: None,
                policy_decision: "diagnostic-observed".to_string(),
            },
        }
    }

    fn diagnostic_path_allowed(allowed_paths: &BTreeSet<PathBuf>, target: &ExecTarget) -> bool {
        let tracee_allowed = allowed_paths.contains(&target.tracee_path);
        let resolved_allowed = allowed_paths.contains(&target.resolved_host_path);
        assert!(target.tracee_path.is_absolute());
        assert!(target.resolved_host_path.is_absolute());
        tracee_allowed || resolved_allowed
    }

    fn classify_notification(
        listener_fd: RawFd,
        policy: &ProtectedExecPolicy,
        notif: &libc::seccomp_notif,
    ) -> SupervisorDecision {
        let syscall_name = syscall_name(notif.data.nr);
        match exec_target(listener_fd, notif) {
            Ok(target) => classify_path(policy, notif.pid, syscall_name, target),
            Err((path, reason)) => denied_event(DeniedEventInput {
                pid: notif.pid,
                syscall_name,
                tracee_path: path,
                resolved_host_path: PathBuf::new(),
                digest_hex: String::new(),
                reason,
                inventory_entry_id: None,
            }),
        }
    }

    fn classify_path(
        policy: &ProtectedExecPolicy,
        pid: u32,
        syscall_name: &'static str,
        target: ExecTarget,
    ) -> SupervisorDecision {
        assert!(target.tracee_path.is_absolute(), "tracee path must be absolute after target resolution");
        assert!(
            target.resolved_host_path.is_absolute(),
            "resolved host path must be absolute after target resolution"
        );
        let digest_hex = match blake3_file_hex(&target.resolved_host_path) {
            Ok(digest) => digest,
            Err(err) => {
                return denied_event(DeniedEventInput {
                    pid,
                    syscall_name,
                    tracee_path: target.tracee_path,
                    resolved_host_path: target.resolved_host_path,
                    digest_hex: String::new(),
                    reason: err.to_string(),
                    inventory_entry_id: None,
                });
            }
        };
        match policy.decide_exec(&ExecRequest {
            path: target.resolved_host_path.clone(),
            digest_hex: digest_hex.clone(),
        }) {
            Ok(decision) => SupervisorDecision {
                allowed: true,
                audit_event: ProtectedSeccompAuditEvent {
                    pid,
                    syscall: syscall_name.to_string(),
                    executable_path: target.resolved_host_path.clone(),
                    tracee_path: target.tracee_path,
                    resolved_host_path: target.resolved_host_path,
                    digest_hex,
                    reason: decision.reason,
                    phase: PHASE_PROTECTED.to_string(),
                    inventory_entry_id: decision.entry_id,
                    policy_decision: "allowed".to_string(),
                },
            },
            Err(err) => denied_event(DeniedEventInput {
                pid,
                syscall_name,
                tracee_path: target.tracee_path,
                resolved_host_path: target.resolved_host_path,
                digest_hex,
                reason: err.to_string(),
                inventory_entry_id: None,
            }),
        }
    }

    struct DeniedEventInput {
        pid: u32,
        syscall_name: &'static str,
        tracee_path: PathBuf,
        resolved_host_path: PathBuf,
        digest_hex: String,
        reason: String,
        inventory_entry_id: Option<String>,
    }

    fn denied_event(input: DeniedEventInput) -> SupervisorDecision {
        SupervisorDecision {
            allowed: false,
            audit_event: ProtectedSeccompAuditEvent {
                pid: input.pid,
                syscall: input.syscall_name.to_string(),
                executable_path: input.resolved_host_path.clone(),
                tracee_path: input.tracee_path,
                resolved_host_path: input.resolved_host_path,
                digest_hex: input.digest_hex,
                reason: input.reason,
                phase: PHASE_PROTECTED.to_string(),
                inventory_entry_id: input.inventory_entry_id,
                policy_decision: "denied".to_string(),
            },
        }
    }

    fn exec_target(listener_fd: RawFd, notif: &libc::seccomp_notif) -> Result<ExecTarget, (PathBuf, String)> {
        validate_execveat_form(notif)?;
        let arg_index = exec_path_arg_index(notif.data.nr)
            .ok_or_else(|| (PathBuf::new(), format!("unexpected syscall number {}", notif.data.nr)))?;
        let address = notif.data.args[arg_index];
        if address == 0 {
            return Err((PathBuf::new(), "exec path pointer is null".to_string()));
        }
        let bytes = read_remote_cstring(RemoteCstringRequest {
            listener_fd,
            pid: notif.pid,
            notification_id: notif.id,
            address,
        })
        .map_err(|err| (PathBuf::new(), err))?;
        if bytes.is_empty() {
            return Err((PathBuf::new(), "empty execveat path is not supported".to_string()));
        }
        let tracee_path = PathBuf::from(OsString::from_vec(bytes));
        if !tracee_path.is_absolute() {
            return Err((tracee_path, "relative exec path cannot be resolved safely".to_string()));
        }
        let resolved_host_path = resolve_tracee_exec_path(listener_fd, notif.pid, notif.id, &tracee_path)
            .map_err(|err| (tracee_path.clone(), err))?;
        assert!(tracee_path.is_absolute());
        assert!(resolved_host_path.is_absolute());
        Ok(ExecTarget {
            tracee_path,
            resolved_host_path,
        })
    }

    fn exec_path_arg_index(syscall_nr: libc::c_int) -> Option<usize> {
        if checked_syscall_number(libc::SYS_execve) == Some(syscall_nr) {
            return Some(EXECVE_PATH_ARG_INDEX);
        }
        if checked_syscall_number(libc::SYS_execveat) == Some(syscall_nr) {
            return Some(EXECVEAT_PATH_ARG_INDEX);
        }
        None
    }

    fn checked_syscall_number(syscall_nr: libc::c_long) -> Option<libc::c_int> {
        libc::c_int::try_from(syscall_nr).ok()
    }

    fn validate_execveat_form(notif: &libc::seccomp_notif) -> Result<(), (PathBuf, String)> {
        let execveat_nr = checked_syscall_number(libc::SYS_execveat)
            .ok_or_else(|| (PathBuf::new(), "execveat syscall number is outside i32".to_string()))?;
        if notif.data.nr != execveat_nr {
            return Ok(());
        }
        let dirfd_raw = notif.data.args[EXECVEAT_DIRFD_ARG_INDEX];
        if is_at_fdcwd_arg(dirfd_raw) {
            return Ok(());
        }
        Err((PathBuf::new(), format!("unsupported execveat dirfd {dirfd_raw}; only AT_FDCWD is supported")))
    }

    fn is_at_fdcwd_arg(raw: u64) -> bool {
        let signed_long = i64::from_ne_bytes(raw.to_ne_bytes());
        let low_int_bits = raw & u64::from(u32::MAX);
        let Ok(unsigned_int) = u32::try_from(low_int_bits) else {
            return false;
        };
        let signed_int = i64::from(i32::from_ne_bytes(unsigned_int.to_ne_bytes()));
        let at_fdcwd = i64::from(libc::AT_FDCWD);
        signed_long == at_fdcwd || signed_int == at_fdcwd
    }

    fn resolve_tracee_exec_path(listener_fd: RawFd, pid: u32, id: u64, tracee_path: &Path) -> Result<PathBuf, String> {
        if !tracee_path.is_absolute() {
            return Err("relative exec path cannot be resolved safely".to_string());
        }
        validate_notification_id(listener_fd, id)?;
        let proc_root_path = proc_root_exec_path(pid, tracee_path)?;
        let file = File::open(&proc_root_path)
            .map_err(|err| format!("open tracee exec path {}: {err}", proc_root_path.display()))?;
        validate_notification_id(listener_fd, id)?;
        let fd_path = PathBuf::from(PROC_FD_PATH_PREFIX).join(file.as_raw_fd().to_string());
        let resolved_host_path = std::fs::read_link(&fd_path)
            .map_err(|err| format!("resolve tracee exec fd {}: {err}", fd_path.display()))?;
        validate_notification_id(listener_fd, id)?;
        if !resolved_host_path.is_absolute() {
            return Err(format!("resolved tracee exec path is not absolute: {}", resolved_host_path.display()));
        }
        Ok(resolved_host_path)
    }

    fn proc_root_exec_path(pid: u32, tracee_path: &Path) -> Result<PathBuf, String> {
        join_tracee_root_path(&PathBuf::from(format!("/proc/{pid}/root")), tracee_path)
    }

    fn join_tracee_root_path(tracee_root: &Path, tracee_path: &Path) -> Result<PathBuf, String> {
        if !tracee_root.is_absolute() {
            return Err(format!("tracee root is not absolute: {}", tracee_root.display()));
        }
        if !tracee_path.is_absolute() {
            return Err("relative exec path cannot be resolved safely".to_string());
        }
        let mut out = tracee_root.to_path_buf();
        for component in tracee_path.components() {
            match component {
                std::path::Component::RootDir | std::path::Component::CurDir => {}
                std::path::Component::Normal(part) => out.push(part),
                std::path::Component::ParentDir => {
                    return Err(format!(
                        "tracee exec path contains unsupported parent component: {}",
                        tracee_path.display()
                    ));
                }
                std::path::Component::Prefix(_) => {
                    return Err(format!("tracee exec path contains unsupported prefix: {}", tracee_path.display()));
                }
            }
        }
        assert!(out.is_absolute());
        assert!(out.starts_with(tracee_root));
        Ok(out)
    }

    struct RemoteCstringRequest {
        listener_fd: RawFd,
        pid: u32,
        notification_id: u64,
        address: u64,
    }

    fn read_remote_cstring(request: RemoteCstringRequest) -> Result<Vec<u8>, String> {
        validate_notification_id(request.listener_fd, request.notification_id)?;
        let mut buf = vec![0_u8; MAX_REMOTE_PATH_BYTES];
        let nread = read_tracee_memory(request.pid, request.address, &mut buf)?;
        validate_notification_id(request.listener_fd, request.notification_id)?;
        if nread == 0 {
            return Err("target exec path read returned EOF".to_string());
        }
        buf.truncate(nread);
        let Some(nul_pos) = buf.iter().position(|byte| *byte == 0) else {
            return Err(format!("target exec path exceeds {MAX_REMOTE_PATH_BYTES} bytes"));
        };
        buf.truncate(nul_pos);
        Ok(buf)
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
        let mem = File::open(format!("/proc/{pid}/mem"))
            .map_err(|error| format!("read target memory: process_vm_readv={process_vm_error}; proc-mem={error}"))?;
        mem.read_at(buffer, address)
            .map_err(|error| format!("read target exec path: process_vm_readv={process_vm_error}; proc-mem={error}"))
    }

    fn validate_notification_id(listener_fd: RawFd, id: u64) -> Result<(), String> {
        let mut id_value = id;
        #[allow(clippy::unnecessary_cast)]
        let rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_ID_VALID as libc::Ioctl, &mut id_value) };
        if rc == 0 {
            return Ok(());
        }
        Err(last_os_error("SECCOMP_IOCTL_NOTIF_ID_VALID"))
    }

    fn send_response(listener_fd: RawFd, id: u64, allowed: bool) -> Result<(), String> {
        let mut resp: libc::seccomp_notif_resp = unsafe { std::mem::zeroed() };
        resp.id = id;
        if allowed {
            resp.flags = u32::try_from(libc::SECCOMP_USER_NOTIF_FLAG_CONTINUE)
                .map_err(|_| "seccomp continue flag exceeds u32".to_string())?;
        } else {
            resp.error = -libc::EACCES;
        }
        #[allow(clippy::unnecessary_cast)]
        let rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_SEND as libc::Ioctl, &mut resp) };
        if rc == 0 {
            return Ok(());
        }
        Err(last_os_error("SECCOMP_IOCTL_NOTIF_SEND"))
    }

    fn syscall_name(syscall_nr: libc::c_int) -> &'static str {
        if checked_syscall_number(libc::SYS_execve) == Some(syscall_nr) {
            return "execve";
        }
        if checked_syscall_number(libc::SYS_execveat) == Some(syscall_nr) {
            return "execveat";
        }
        "unknown"
    }

    fn last_os_error(context: &str) -> String {
        format!("{context}: {}", io::Error::last_os_error())
    }

    #[cfg(test)]
    mod tests {
        use std::ffi::CString;
        use std::path::Path;
        use std::process::Command;

        use super::*;
        use crate::protected_exec::DigestSpec;
        use crate::protected_exec::ExecutableSeedEntry;
        use crate::protected_exec::Stage0Inventory;

        const CHILD_MODE_VAR: &str = "CRUNCH_TEST_SECCOMP_CHILD_MODE";
        const LISTENER_ISOLATION_CHILD_MODE: &str = "listener-isolation";
        const FRESH_LISTENER_WORKER_COUNT: usize = 2;
        const _: () = assert!(FRESH_LISTENER_WORKER_COUNT > 1);
        const AUDIT_FLUSH_WAIT_MS: u64 = 50;
        const DIAGNOSTIC_TEST_EVENT_COUNT: usize = 2;
        const ORPHAN_EXEC_DELAY_US: libc::useconds_t = 100_000;

        fn current_exe_policy(digest_hex: String) -> ProtectedExecPolicy {
            let current_exe = std::env::current_exe().unwrap();
            let shell_placeholder = current_exe.with_extension("sandbox-shell-placeholder");
            let inventory = Stage0Inventory {
                executable_entries: vec![
                    seed_entry("sandbox-entry", "sandbox-entry", &current_exe, digest_hex.clone(), true),
                    seed_entry("sandbox-shell", "sandbox-shell", &shell_placeholder, digest_hex, true),
                ],
                source_entries: Vec::new(),
            };
            ProtectedExecPolicy::from_inventory(inventory).unwrap()
        }

        fn seed_entry(id: &str, role: &str, path: &Path, digest_hex: String, required: bool) -> ExecutableSeedEntry {
            ExecutableSeedEntry {
                schema_version: "host-tool-free-stage0-v1".to_string(),
                id: id.to_string(),
                role: role.to_string(),
                phase: "protected".to_string(),
                executable_path: path.to_path_buf(),
                digest: DigestSpec {
                    algorithm: "blake3".to_string(),
                    hex: digest_hex,
                    interoperability_reason: None,
                },
                version_evidence: Some(crate::protected_exec::bounded_version_evidence(
                    vec![path.display().to_string(), "--version".to_string()],
                    format!("{id} seccomp-test-version\n"),
                    0,
                )),
                provenance_category: "test-fixture".to_string(),
                provenance: "seccomp unit test".to_string(),
                allowed_reason: format!("allow {id}"),
                owner: "bootstrap".to_string(),
                required,
            }
        }

        #[test]
        fn seccomp_supervisor_unsupported_arch_fails_closed_before_filter_install() {
            let err = require_supported_audit_arch(None).unwrap_err();

            assert_eq!(
                err.to_string(),
                "protected exec supervisor unsupported: unsupported Linux audit architecture for exec supervisor"
            );
        }

        #[test]
        fn tracee_root_join_maps_sandbox_bin_sh_inside_root() {
            let temp = tempfile::tempdir().unwrap();
            let bin_dir = temp.path().join("bin");
            std::fs::create_dir_all(&bin_dir).unwrap();
            let shell = bin_dir.join("sh");
            std::fs::write(&shell, b"#!/bin/sh\nexit 0\n").unwrap();

            let joined = join_tracee_root_path(temp.path(), Path::new("/bin/sh")).unwrap();

            assert_eq!(joined, shell);
            assert!(joined.starts_with(temp.path()));
        }

        #[test]
        fn tracee_root_join_rejects_ambiguous_relative_and_parent_paths() {
            let temp = tempfile::tempdir().unwrap();
            let relative = join_tracee_root_path(temp.path(), Path::new("bin/sh")).unwrap_err();
            let parent = join_tracee_root_path(temp.path(), Path::new("/../bin/sh")).unwrap_err();

            assert!(relative.contains("relative exec path"), "reason: {relative}");
            assert!(parent.contains("parent component"), "reason: {parent}");
        }

        #[test]
        fn diagnostic_exec_path_validation_is_bounded_and_absolute() {
            let empty = BTreeSet::new();
            let relative = BTreeSet::from([PathBuf::from("relative-tool")]);
            let excessive = (0..=DIAGNOSTIC_EXEC_PATH_COUNT_MAX)
                .map(|index| PathBuf::from(format!("/diagnostic/tool-{index}")))
                .collect::<BTreeSet<_>>();
            assert!(validate_diagnostic_paths(&empty).is_err());
            assert!(validate_diagnostic_paths(&relative).is_err());
            assert!(validate_diagnostic_paths(&excessive).is_err());
            assert_eq!(excessive.len(), DIAGNOSTIC_EXEC_PATH_COUNT_MAX.saturating_add(1));
        }

        #[test]
        fn diagnostic_observer_records_exact_paths_without_granting_authority() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("diagnostic-observer") {
                run_diagnostic_observer_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg(
                    "protected_exec_seccomp::linux::tests::diagnostic_observer_records_exact_paths_without_granting_authority",
                )
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "diagnostic-observer")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_allows_declared_execve() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("allow") {
                run_allow_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_allows_declared_execve")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "allow")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_listeners_require_distinct_fresh_worker_threads() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some(LISTENER_ISOLATION_CHILD_MODE) {
                run_listener_isolation_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_listeners_require_distinct_fresh_worker_threads")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, LISTENER_ISOLATION_CHILD_MODE)
                .output()
                .unwrap();

            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "stderr={}", String::from_utf8_lossy(&output.stderr));
        }

        #[test]
        fn seccomp_supervisor_denies_digest_mismatch_before_execve() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("deny") {
                run_deny_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_digest_mismatch_before_execve")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "deny")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_denies_undeclared_host_bwrap_before_execve() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("host-bwrap") {
                run_denied_host_bwrap_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_undeclared_host_bwrap_before_execve")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "host-bwrap")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_allows_declared_execveat_descendant() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("execveat") {
                run_execveat_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_allows_declared_execveat_descendant")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "execveat")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn adopted_descendant_reaper_rejects_failed_exit() {
            const REAPER_FAILURE_MODE: &str = "reaper-failed-descendant";
            const FAILED_EXIT_STATUS: libc::c_int = 7;
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some(REAPER_FAILURE_MODE) {
                let pid = unsafe { libc::fork() };
                assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
                if pid == 0 {
                    unsafe { libc::_exit(FAILED_EXIT_STATUS) };
                }
                let error = reap_adopted_exec_descendants().unwrap_err();
                assert!(error.to_string().contains("failed with wait status"));
                assert!(!error.to_string().contains("timed out"));
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::adopted_descendant_reaper_rejects_failed_exit")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, REAPER_FAILURE_MODE)
                .output()
                .unwrap();
            assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
            assert!(output.stdout.is_empty() || String::from_utf8_lossy(&output.stdout).contains("test result: ok"));
        }

        #[test]
        fn seccomp_supervisor_denies_orphan_exec_without_subreaper_adoption() {
            match std::env::var(CHILD_MODE_VAR).ok().as_deref() {
                Some("orphan-without-subreaper-parent") => {
                    run_orphan_without_subreaper_parent();
                    return;
                }
                Some("orphan-without-subreaper-middle") => {
                    run_deep_middle_child();
                    return;
                }
                _ => {}
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg(
                    "protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_orphan_exec_without_subreaper_adoption",
                )
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "orphan-without-subreaper-parent")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_reads_deep_descendant_exec_path() {
            match std::env::var(CHILD_MODE_VAR).ok().as_deref() {
                Some("deep-parent") => {
                    run_deep_parent_child();
                    return;
                }
                Some("deep-middle") => {
                    run_deep_middle_child();
                    return;
                }
                _ => {}
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_reads_deep_descendant_exec_path")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "deep-parent")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_resolves_symlink_before_digesting() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("symlink-resolution") {
                run_symlink_resolution_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_resolves_symlink_before_digesting")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "symlink-resolution")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_denies_relative_exec_path() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("relative-path") {
                run_relative_path_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_relative_exec_path")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "relative-path")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_denies_unreadable_exec_path() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("unreadable-path") {
                run_unreadable_path_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_unreadable_exec_path")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "unreadable-path")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[test]
        fn seccomp_supervisor_denies_execveat_non_fdcwd_dirfd() {
            if std::env::var(CHILD_MODE_VAR).ok().as_deref() == Some("execveat-dirfd") {
                run_execveat_dirfd_child();
                return;
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_execveat_non_fdcwd_dirfd")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "execveat-dirfd")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        fn run_diagnostic_observer_child() {
            let current_exe = std::env::current_exe().unwrap();
            let temp = tempfile::tempdir().unwrap();
            let declared = temp.path().join("declared-exec");
            std::os::unix::fs::symlink(&current_exe, &declared).unwrap();
            let undeclared = temp.path().join("undeclared-exec");
            std::fs::copy(&current_exe, &undeclared).unwrap();
            make_executable(&undeclared);
            let observer = install_current_thread_diagnostic_exec_observer(BTreeSet::from([declared.clone()])).unwrap();
            let status = Command::new(&declared).arg("--help").status().unwrap();
            let denied = Command::new(&undeclared).arg("--help").status().unwrap_err();
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
            let events = observer.audit_events();
            assert!(status.success());
            assert_eq!(denied.kind(), std::io::ErrorKind::PermissionDenied);
            assert_eq!(events.len(), DIAGNOSTIC_TEST_EVENT_COUNT);
            assert_eq!(events[0].policy_decision, "diagnostic-observed");
            assert_eq!(events[0].phase, PHASE_DIAGNOSTIC);
            assert!(events[0].inventory_entry_id.is_none());
            assert_eq!(events[1].policy_decision, "denied");
            assert!(events[1].reason.contains("exact diagnostic exec inventory"));
        }

        fn run_listener_isolation_child() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            for _ in 0..FRESH_LISTENER_WORKER_COUNT {
                let worker_digest = digest_hex.clone();
                std::thread::spawn(move || {
                    let supervisor = install_current_thread_exec_supervisor(current_exe_policy(worker_digest)).unwrap();
                    assert!(supervisor.listener_fd() >= 0);
                    assert!(supervisor.audit_events().is_empty());
                })
                .join()
                .unwrap();
            }
            let same_thread_digest = digest_hex.clone();
            std::thread::spawn(move || {
                let first =
                    install_current_thread_exec_supervisor(current_exe_policy(same_thread_digest.clone())).unwrap();
                let second =
                    install_current_thread_exec_supervisor(current_exe_policy(same_thread_digest)).unwrap_err();
                assert!(first.listener_fd() >= 0);
                assert!(matches!(second, ProtectedSeccompError::Install(_)));
                assert!(second.to_string().contains("Device or resource busy"));
            })
            .join()
            .unwrap();
            let status = Command::new(&current_exe).arg("--help").status().unwrap();

            assert!(status.success());
        }

        fn run_allow_child() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let status = Command::new(&current_exe).arg("--help").status().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            let events = supervisor.audit_events();
            assert!(status.success());
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "allowed");
            assert_eq!(events[0].syscall, "execve");
            assert_eq!(events[0].executable_path, current_exe);
            assert_eq!(events[0].inventory_entry_id.as_deref(), Some("sandbox-entry"));
        }

        fn run_deny_child() {
            let current_exe = std::env::current_exe().unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy("0".repeat(64))).unwrap();
            let err = Command::new(&current_exe).arg("--help").status().unwrap_err();
            std::thread::sleep(std::time::Duration::from_millis(50));
            let events = supervisor.audit_events();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert!(events[0].reason.contains("digest mismatch"));
        }

        fn run_denied_host_bwrap_child() {
            let temp = tempfile::tempdir().unwrap();
            let bwrap = temp.path().join("bwrap");
            std::fs::write(&bwrap, b"#!/bin/sh\nexit 0\n").unwrap();
            make_executable(&bwrap);
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let err = Command::new(&bwrap).status().unwrap_err();
            std::thread::sleep(std::time::Duration::from_millis(50));
            let events = supervisor.audit_events();
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert_eq!(events[0].syscall, "execve");
            assert_eq!(events[0].executable_path, bwrap);
            assert!(events[0].inventory_entry_id.is_none());
            assert!(events[0].reason.contains("declared"), "reason: {}", events[0].reason);
        }

        #[cfg(unix)]
        fn make_executable(path: &Path) {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).unwrap();
        }

        fn run_execveat_child() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
            if pid == 0 {
                execveat_current_exe_help(&current_exe);
            }
            let mut status: libc::c_int = 0;
            let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
            assert_eq!(waited, pid, "waitpid failed: {}", std::io::Error::last_os_error());
            std::thread::sleep(std::time::Duration::from_millis(50));
            let events = supervisor.audit_events();
            assert!(libc::WIFEXITED(status), "child status was {status}");
            assert_eq!(libc::WEXITSTATUS(status), 0);
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "allowed");
            assert_eq!(events[0].syscall, "execveat");
            assert_eq!(events[0].executable_path, current_exe);
            assert_eq!(events[0].inventory_entry_id.as_deref(), Some("sandbox-entry"));
        }

        fn run_orphan_without_subreaper_parent() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let policy = current_exe_policy(digest_hex);
            verify_notification_sizes().unwrap();
            set_no_new_privileges().unwrap();
            let listener_fd = install_exec_filter().unwrap();
            let audit_events = Arc::new(Mutex::new(Vec::new()));
            let shared_policy = Arc::new(RwLock::new(policy));
            spawn_supervisor_thread(listener_fd, shared_policy, audit_events.clone()).unwrap();
            let status = Command::new(&current_exe)
                .arg("--exact")
                .arg(
                    "protected_exec_seccomp::linux::tests::seccomp_supervisor_denies_orphan_exec_without_subreaper_adoption",
                )
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "orphan-without-subreaper-middle")
                .status()
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS.saturating_mul(4)));
            let events = audit_events.lock().unwrap().clone();
            assert!(status.success());
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].policy_decision, "allowed");
            assert_eq!(events[1].policy_decision, "denied");
            assert!(events[1].reason.contains("read target memory"), "reason: {}", events[1].reason);
        }

        fn run_deep_parent_child() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let status = Command::new(&current_exe)
                .arg("--exact")
                .arg("protected_exec_seccomp::linux::tests::seccomp_supervisor_reads_deep_descendant_exec_path")
                .arg("--nocapture")
                .env(CHILD_MODE_VAR, "deep-middle")
                .status()
                .unwrap();
            let reaped_count = reap_adopted_exec_descendants().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
            let events = supervisor.audit_events();
            assert!(status.success());
            assert_eq!(reaped_count, 1);
            assert_eq!(events.len(), 2);
            assert!(events.iter().all(|event| event.policy_decision == "allowed"));
        }

        fn run_deep_middle_child() {
            let current_exe = std::env::current_exe().unwrap();
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
            if pid == 0 {
                unsafe {
                    libc::usleep(ORPHAN_EXEC_DELAY_US);
                }
                execve_current_exe_help(&current_exe);
            }
        }

        fn run_symlink_resolution_child() {
            let temp = tempfile::tempdir().unwrap();
            let bin_dir = temp.path().join("bin");
            std::fs::create_dir_all(&bin_dir).unwrap();
            let current_exe = std::env::current_exe().unwrap();
            let symlink_path = bin_dir.join("sh");
            std::os::unix::fs::symlink(&current_exe, &symlink_path).unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();

            let status = Command::new(&symlink_path).arg("--help").status().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
            let events = supervisor.audit_events();

            assert!(status.success());
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "allowed");
            assert_eq!(events[0].tracee_path, symlink_path);
            assert_eq!(events[0].resolved_host_path, current_exe);
            assert_eq!(events[0].executable_path, events[0].resolved_host_path);
        }

        fn run_relative_path_child() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
            if pid == 0 {
                execve_relative_path();
            }
            assert_child_exit(pid, libc::EACCES);
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
            let events = supervisor.audit_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert_eq!(events[0].tracee_path, PathBuf::from("./missing-relative-exec"));
            assert!(events[0].reason.contains("relative exec path"), "reason: {}", events[0].reason);
        }

        fn run_unreadable_path_child() {
            let temp = tempfile::tempdir().unwrap();
            let directory_exec = temp.path().join("not-a-regular-exec");
            std::fs::create_dir(&directory_exec).unwrap();
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
            if pid == 0 {
                execve_absolute_path(&directory_exec);
            }
            assert_child_exit(pid, libc::EACCES);
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
            let events = supervisor.audit_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert_eq!(events[0].tracee_path, directory_exec);
            assert!(!events[0].reason.is_empty());
        }

        fn run_execveat_dirfd_child() {
            let current_exe = std::env::current_exe().unwrap();
            let digest_hex = blake3_file_hex(&current_exe).unwrap();
            let supervisor = install_current_thread_exec_supervisor(current_exe_policy(digest_hex)).unwrap();
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
            if pid == 0 {
                execveat_with_dirfd(&current_exe);
            }
            assert_child_exit(pid, libc::EACCES);
            std::thread::sleep(std::time::Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
            let events = supervisor.audit_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].policy_decision, "denied");
            assert!(events[0].reason.contains("execveat dirfd"), "reason: {}", events[0].reason);
            assert!(events[0].reason.contains("AT_FDCWD"), "reason: {}", events[0].reason);
        }

        fn assert_child_exit(pid: libc::pid_t, expected_code: i32) {
            let mut status: libc::c_int = 0;
            let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
            assert_eq!(waited, pid, "waitpid failed: {}", std::io::Error::last_os_error());
            assert!(libc::WIFEXITED(status), "child status was {status}");
            assert_eq!(libc::WEXITSTATUS(status), expected_code);
        }

        fn execve_relative_path() -> ! {
            let path = CString::new("./missing-relative-exec").unwrap();
            execve_raw_path(path.as_ptr())
        }

        fn execve_absolute_path(path: &Path) -> ! {
            let path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
            execve_raw_path(path.as_ptr())
        }

        fn execve_raw_path(path: *const libc::c_char) -> ! {
            let argv0 = CString::new("crunch-seccomp-test").unwrap();
            let argv = [argv0.as_ptr(), std::ptr::null()];
            let envp = [std::ptr::null::<libc::c_char>()];
            unsafe {
                libc::execve(path, argv.as_ptr(), envp.as_ptr());
                libc::_exit(std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EINVAL));
            }
        }

        fn execveat_with_dirfd(current_exe: &Path) -> ! {
            use std::os::fd::AsRawFd;
            let parent = current_exe.parent().unwrap();
            let dir = std::fs::File::open(parent).unwrap();
            let file_name = current_exe.file_name().unwrap().as_encoded_bytes();
            let path = CString::new(file_name).unwrap();
            let argv0 = CString::new("crunch-seccomp-test").unwrap();
            let arg_help = CString::new("--help").unwrap();
            let argv = [argv0.as_ptr(), arg_help.as_ptr(), std::ptr::null()];
            let envp = [std::ptr::null::<libc::c_char>()];
            unsafe {
                libc::syscall(libc::SYS_execveat, dir.as_raw_fd(), path.as_ptr(), argv.as_ptr(), envp.as_ptr(), 0);
                libc::_exit(std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EINVAL));
            }
        }

        fn execve_current_exe_help(current_exe: &Path) -> ! {
            let path = CString::new(current_exe.as_os_str().as_encoded_bytes()).unwrap();
            let argv0 = CString::new("crunch-seccomp-test").unwrap();
            let arg_help = CString::new("--help").unwrap();
            let argv = [argv0.as_ptr(), arg_help.as_ptr(), std::ptr::null()];
            let envp = [std::ptr::null::<libc::c_char>()];
            unsafe {
                libc::execve(path.as_ptr(), argv.as_ptr(), envp.as_ptr());
                libc::_exit(127);
            }
        }

        fn execveat_current_exe_help(current_exe: &Path) -> ! {
            let path = CString::new(current_exe.as_os_str().as_encoded_bytes()).unwrap();
            let argv0 = CString::new("crunch-seccomp-test").unwrap();
            let arg_help = CString::new("--help").unwrap();
            let argv = [argv0.as_ptr(), arg_help.as_ptr(), std::ptr::null()];
            let envp = [std::ptr::null::<libc::c_char>()];
            unsafe {
                libc::syscall(libc::SYS_execveat, libc::AT_FDCWD, path.as_ptr(), argv.as_ptr(), envp.as_ptr(), 0);
                libc::_exit(127);
            }
        }
    }
}

#[allow(unused_imports)]
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(not(target_os = "linux"))]
mod non_linux {
    use crate::protected_exec::ProtectedExecPolicy;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum ProtectedSeccompError {
        Unsupported(String),
    }

    impl std::fmt::Display for ProtectedSeccompError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unsupported(message) => write!(f, "protected exec supervisor unsupported: {message}"),
            }
        }
    }

    impl std::error::Error for ProtectedSeccompError {}

    #[derive(Debug)]
    pub struct ProtectedSeccompSupervisor;

    pub fn install_current_thread_exec_supervisor(
        _policy: ProtectedExecPolicy,
    ) -> Result<ProtectedSeccompSupervisor, ProtectedSeccompError> {
        Err(ProtectedSeccompError::Unsupported("seccomp user notification is Linux-only".to_string()))
    }
}

#[cfg(not(target_os = "linux"))]
pub use non_linux::*;
