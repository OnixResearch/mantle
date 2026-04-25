#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::OsString;
    use std::fs::File;
    use std::io;
    use std::os::fd::RawFd;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::FileExt;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;

    use crate::protected_exec::ExecRequest;
    use crate::protected_exec::PHASE_PROTECTED;
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
        listener_fd: RawFd,
    }

    impl ProtectedSeccompSupervisor {
        pub fn audit_events(&self) -> Vec<ProtectedSeccompAuditEvent> {
            self.audit_events.lock().expect("seccomp audit mutex poisoned").clone()
        }

        pub fn listener_fd(&self) -> RawFd {
            self.listener_fd
        }
    }

    pub fn install_current_thread_exec_supervisor(
        policy: ProtectedExecPolicy,
    ) -> Result<ProtectedSeccompSupervisor, ProtectedSeccompError> {
        verify_notification_sizes()?;
        set_no_new_privileges()?;
        let listener_fd = install_exec_filter()?;
        let audit_events = Arc::new(Mutex::new(Vec::new()));
        spawn_supervisor_thread(listener_fd, policy, audit_events.clone())?;
        Ok(ProtectedSeccompSupervisor {
            audit_events,
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
        let arch = current_audit_arch().ok_or_else(|| {
            ProtectedSeccompError::Unsupported("unsupported Linux audit architecture for exec supervisor".to_string())
        })?;
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
        policy: ProtectedExecPolicy,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) -> Result<(), ProtectedSeccompError> {
        thread::Builder::new()
            .name("crunch-protected-exec-supervisor".to_string())
            .spawn(move || supervisor_loop(listener_fd, policy, audit_events))
            .map(|_| ())
            .map_err(|err| ProtectedSeccompError::Supervisor(format!("spawning supervisor thread: {err}")))
    }

    fn supervisor_loop(
        listener_fd: RawFd,
        policy: ProtectedExecPolicy,
        audit_events: Arc<Mutex<Vec<ProtectedSeccompAuditEvent>>>,
    ) {
        loop {
            let mut notif: libc::seccomp_notif = unsafe { std::mem::zeroed() };
            let recv_rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_RECV, &mut notif) };
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
            let decision = classify_notification(listener_fd, &policy, &notif);
            audit_events.lock().expect("seccomp audit mutex poisoned").push(decision.audit_event);
            let _ = send_response(listener_fd, notif.id, decision.allowed);
        }
    }

    struct SupervisorDecision {
        allowed: bool,
        audit_event: ProtectedSeccompAuditEvent,
    }

    fn classify_notification(
        listener_fd: RawFd,
        policy: &ProtectedExecPolicy,
        notif: &libc::seccomp_notif,
    ) -> SupervisorDecision {
        let syscall_name = syscall_name(notif.data.nr);
        match exec_path(listener_fd, notif) {
            Ok(path) => classify_path(policy, notif.pid, syscall_name, path),
            Err((path, reason)) => denied_event(notif.pid, syscall_name, path, String::new(), reason, None),
        }
    }

    fn classify_path(
        policy: &ProtectedExecPolicy,
        pid: u32,
        syscall_name: &'static str,
        path: PathBuf,
    ) -> SupervisorDecision {
        if !path.is_absolute() {
            return denied_event(
                pid,
                syscall_name,
                path,
                String::new(),
                "relative exec path cannot be resolved safely".to_string(),
                None,
            );
        }
        let digest_hex = match blake3_file_hex(&path) {
            Ok(digest) => digest,
            Err(err) => {
                return denied_event(pid, syscall_name, path, String::new(), err.to_string(), None);
            }
        };
        match policy.decide_exec(&ExecRequest {
            path: path.clone(),
            digest_hex: digest_hex.clone(),
        }) {
            Ok(decision) => SupervisorDecision {
                allowed: true,
                audit_event: ProtectedSeccompAuditEvent {
                    pid,
                    syscall: syscall_name.to_string(),
                    executable_path: path,
                    digest_hex,
                    reason: decision.reason,
                    phase: PHASE_PROTECTED.to_string(),
                    inventory_entry_id: decision.entry_id,
                    policy_decision: "allowed".to_string(),
                },
            },
            Err(err) => denied_event(pid, syscall_name, path, digest_hex, err.to_string(), inventory_entry_id(&err)),
        }
    }

    fn denied_event(
        pid: u32,
        syscall_name: &'static str,
        path: PathBuf,
        digest_hex: String,
        reason: String,
        inventory_entry_id: Option<String>,
    ) -> SupervisorDecision {
        SupervisorDecision {
            allowed: false,
            audit_event: ProtectedSeccompAuditEvent {
                pid,
                syscall: syscall_name.to_string(),
                executable_path: path,
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

    fn exec_path(listener_fd: RawFd, notif: &libc::seccomp_notif) -> Result<PathBuf, (PathBuf, String)> {
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
        Ok(PathBuf::from(OsString::from_vec(bytes)))
    }

    fn exec_path_arg_index(syscall_nr: libc::c_int) -> Option<usize> {
        if syscall_nr == libc::SYS_execve as libc::c_int {
            return Some(0);
        }
        if syscall_nr == libc::SYS_execveat as libc::c_int {
            return Some(1);
        }
        None
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
        let rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_ID_VALID, &mut id_value) };
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
        let rc = unsafe { libc::ioctl(listener_fd, SECCOMP_IOCTL_NOTIF_SEND, &mut resp) };
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
