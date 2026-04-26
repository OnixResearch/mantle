#[cfg(target_os = "linux")]
mod linux {
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
    const FILTER_INSTRUCTION_COUNT: u16 = 8;
    const EXECVE_PATH_ARG_INDEX: usize = 0;
    const EXECVEAT_DIRFD_ARG_INDEX: usize = 0;
    const EXECVEAT_PATH_ARG_INDEX: usize = 1;
    const PROC_FD_PATH_PREFIX: &str = "/proc/self/fd";

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
    pub struct ProtectedSeccompSupervisor {
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
        shared_policy: Arc<RwLock<ProtectedExecPolicy>>,
        listener_fd: RawFd,
    }

    impl ProtectedSeccompSupervisor {
        pub fn audit_events(&self) -> Vec<ProtectedSeccompAuditEvent> {
            self.audit_events.lock().expect("seccomp audit mutex poisoned").clone()
        }

        pub fn listener_fd(&self) -> RawFd {
            self.listener_fd
        }

        pub fn promote_verified_output(
            &self,
            source_entry_id: &str,
            extraction_rules: &[String],
            executables: &[PromotedExecutable],
        ) -> Result<OutputPromotionRecord, ProtectedExecError> {
            self.shared_policy
                .write()
                .expect("seccomp policy rwlock poisoned")
                .promote_verified_output(source_entry_id, extraction_rules, executables)
        }
    }

    pub fn install_current_thread_exec_supervisor(
        policy: ProtectedExecPolicy,
    ) -> Result<ProtectedSeccompSupervisor, ProtectedSeccompError> {
        verify_notification_sizes()?;
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
        let mut sizes: libc::seccomp_notif_sizes = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::syscall(libc::SYS_seccomp, libc::SECCOMP_GET_NOTIF_SIZES, 0, &mut sizes) };
        if rc == 0 {
            return Ok(());
        }
        Err(ProtectedSeccompError::Unsupported(last_os_error("SECCOMP_GET_NOTIF_SIZES")))
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
        let mut filter = exec_filter(arch);
        let mut program = libc::sock_fprog {
            len: FILTER_INSTRUCTION_COUNT,
            filter: filter.as_mut_ptr(),
        };
        let fd = unsafe {
            libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_SET_MODE_FILTER,
                libc::SECCOMP_FILTER_FLAG_NEW_LISTENER,
                &mut program,
            )
        };
        if fd >= 0 {
            return Ok(fd as RawFd);
        }
        Err(ProtectedSeccompError::Install(last_os_error("SECCOMP_SET_MODE_FILTER")))
    }

    fn exec_filter(arch: u32) -> [libc::sock_filter; FILTER_INSTRUCTION_COUNT as usize] {
        [
            bpf_stmt(libc::BPF_LD | libc::BPF_W | libc::BPF_ABS, SECCOMP_DATA_ARCH_OFFSET),
            bpf_jump(libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K, arch, 1, 0),
            bpf_stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_KILL_PROCESS),
            bpf_stmt(libc::BPF_LD | libc::BPF_W | libc::BPF_ABS, SECCOMP_DATA_NR_OFFSET),
            bpf_jump(libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K, libc::SYS_execve as u32, 1, 0),
            bpf_jump(libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K, libc::SYS_execveat as u32, 0, 1),
            bpf_stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_USER_NOTIF),
            bpf_stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_ALLOW),
        ]
    }

    fn bpf_stmt(code: u32, k: u32) -> libc::sock_filter {
        libc::sock_filter {
            code: code as u16,
            jt: 0,
            jf: 0,
            k,
        }
    }

    fn bpf_jump(code: u32, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
        libc::sock_filter {
            code: code as u16,
            jt,
            jf,
            k,
        }
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
        loop {
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
            let policy = shared_policy.read().expect("seccomp policy rwlock poisoned");
            let decision = classify_notification(listener_fd, &policy, &notif);
            drop(policy);
            audit_events.lock().expect("seccomp audit mutex poisoned").push(decision.audit_event);
            let _ = send_response(listener_fd, notif.id, decision.allowed);
        }
    }

    struct SupervisorDecision {
        allowed: bool,
        audit_event: ProtectedSeccompAuditEvent,
    }

    struct ExecTarget {
        tracee_path: PathBuf,
        resolved_host_path: PathBuf,
    }

    fn classify_notification(
        listener_fd: RawFd,
        policy: &ProtectedExecPolicy,
        notif: &libc::seccomp_notif,
    ) -> SupervisorDecision {
        let syscall_name = syscall_name(notif.data.nr);
        match exec_target(listener_fd, notif) {
            Ok(target) => classify_path(policy, notif.pid, syscall_name, target),
            Err((path, reason)) => {
                denied_event(notif.pid, syscall_name, path.clone(), PathBuf::new(), String::new(), reason, None)
            }
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
                return denied_event(
                    pid,
                    syscall_name,
                    target.tracee_path,
                    target.resolved_host_path,
                    String::new(),
                    err.to_string(),
                    None,
                );
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
            Err(err) => denied_event(
                pid,
                syscall_name,
                target.tracee_path,
                target.resolved_host_path,
                digest_hex,
                err.to_string(),
                inventory_entry_id(&err),
            ),
        }
    }

    fn denied_event(
        pid: u32,
        syscall_name: &'static str,
        tracee_path: PathBuf,
        resolved_host_path: PathBuf,
        digest_hex: String,
        reason: String,
        inventory_entry_id: Option<String>,
    ) -> SupervisorDecision {
        SupervisorDecision {
            allowed: false,
            audit_event: ProtectedSeccompAuditEvent {
                pid,
                syscall: syscall_name.to_string(),
                executable_path: resolved_host_path.clone(),
                tracee_path,
                resolved_host_path,
                digest_hex,
                reason,
                phase: PHASE_PROTECTED.to_string(),
                inventory_entry_id,
                policy_decision: "denied".to_string(),
            },
        }
    }

    fn inventory_entry_id(err: &ProtectedExecError) -> Option<String> {
        match err {
            ProtectedExecError::DigestMismatch { .. } => None,
            _ => None,
        }
    }

    fn exec_target(listener_fd: RawFd, notif: &libc::seccomp_notif) -> Result<ExecTarget, (PathBuf, String)> {
        validate_execveat_form(notif)?;
        let arg_index = exec_path_arg_index(notif.data.nr)
            .ok_or_else(|| (PathBuf::new(), format!("unexpected syscall number {}", notif.data.nr)))?;
        let addr = notif.data.args[arg_index];
        if addr == 0 {
            return Err((PathBuf::new(), "exec path pointer is null".to_string()));
        }
        let bytes = read_remote_cstring(listener_fd, notif.pid, notif.id, addr).map_err(|err| (PathBuf::new(), err))?;
        if bytes.is_empty() {
            return Err((PathBuf::new(), "empty execveat path is not supported".to_string()));
        }
        let tracee_path = PathBuf::from(OsString::from_vec(bytes));
        if !tracee_path.is_absolute() {
            return Err((tracee_path, "relative exec path cannot be resolved safely".to_string()));
        }
        let resolved_host_path = resolve_tracee_exec_path(listener_fd, notif.pid, notif.id, &tracee_path)
            .map_err(|err| (tracee_path.clone(), err))?;
        Ok(ExecTarget {
            tracee_path,
            resolved_host_path,
        })
    }

    fn exec_path_arg_index(syscall_nr: libc::c_int) -> Option<usize> {
        if syscall_nr == libc::SYS_execve as libc::c_int {
            return Some(EXECVE_PATH_ARG_INDEX);
        }
        if syscall_nr == libc::SYS_execveat as libc::c_int {
            return Some(EXECVEAT_PATH_ARG_INDEX);
        }
        None
    }

    fn validate_execveat_form(notif: &libc::seccomp_notif) -> Result<(), (PathBuf, String)> {
        if notif.data.nr != libc::SYS_execveat as libc::c_int {
            return Ok(());
        }
        let dirfd_raw = notif.data.args[EXECVEAT_DIRFD_ARG_INDEX];
        if is_at_fdcwd_arg(dirfd_raw) {
            return Ok(());
        }
        Err((PathBuf::new(), format!("unsupported execveat dirfd {dirfd_raw}; only AT_FDCWD is supported")))
    }

    fn is_at_fdcwd_arg(raw: u64) -> bool {
        let signed_long = raw as i64;
        let signed_int = raw as u32 as i32 as i64;
        let at_fdcwd = libc::AT_FDCWD as i64;
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
        Ok(out)
    }

    fn read_remote_cstring(listener_fd: RawFd, pid: u32, id: u64, addr: u64) -> Result<Vec<u8>, String> {
        validate_notification_id(listener_fd, id)?;
        let mem = File::open(format!("/proc/{pid}/mem")).map_err(|err| format!("open target memory: {err}"))?;
        validate_notification_id(listener_fd, id)?;
        let mut buf = vec![0_u8; MAX_REMOTE_PATH_BYTES];
        let nread = mem.read_at(&mut buf, addr).map_err(|err| format!("read target exec path: {err}"))?;
        validate_notification_id(listener_fd, id)?;
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
            resp.flags = libc::SECCOMP_USER_NOTIF_FLAG_CONTINUE as u32;
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
        if syscall_nr == libc::SYS_execve as libc::c_int {
            return "execve";
        }
        if syscall_nr == libc::SYS_execveat as libc::c_int {
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
        const AUDIT_FLUSH_WAIT_MS: u64 = 50;

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
