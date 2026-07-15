#[cfg(target_os = "linux")]
mod linux {
    use std::fs::File;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::fd::RawFd;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    const AUDIT_ARCH_X86_64: u32 = 0xC000_003E;
    const AUDIT_ARCH_AARCH64: u32 = 0xC000_00B7;
    const SECCOMP_DATA_NR_OFFSET_BYTES: u32 = 0;
    const SECCOMP_DATA_ARCH_OFFSET_BYTES: u32 = 4;
    const FILTER_FIXED_INSTRUCTION_COUNT: usize = 5;
    const FILTER_INSTRUCTIONS_PER_DENIED_SYSCALL: usize = 2;
    const FILTER_IDENTITY_PREFIX: &str = "mantle-clock-syscall-deny-v1";
    const FILTER_DENIAL_ERRNO: u32 = libc::EPERM.unsigned_abs();
    const HEX_CHARS_PER_BYTE: usize = 2;

    #[derive(Debug)]
    pub struct ProofClockSeccompFilter {
        file: File,
        identity: String,
    }

    impl ProofClockSeccompFilter {
        pub fn fd(&self) -> RawFd {
            let fd = self.file.as_raw_fd();
            assert!(fd >= 0, "proof clock seccomp filter fd must be valid");
            assert!(!self.identity.is_empty(), "proof clock seccomp filter identity must be present");
            fd
        }

        pub fn identity(&self) -> &str {
            assert!(!self.identity.is_empty(), "proof clock seccomp filter identity must be present");
            assert!(self.identity.starts_with(FILTER_IDENTITY_PREFIX));
            &self.identity
        }

        pub fn configure_inheritance(&self, command: &mut Command) {
            let seccomp_fd = self.fd();
            assert!(seccomp_fd >= 0, "proof seccomp fd must be valid");
            assert_ne!(seccomp_fd, libc::STDIN_FILENO, "proof seccomp fd must not alias stdin");
            // SAFETY: `fcntl` is async-signal-safe, and the closure only changes this inherited fd's
            // close-on-exec flag.
            unsafe {
                command.pre_exec(move || {
                    let current_flags = libc::fcntl(seccomp_fd, libc::F_GETFD);
                    if current_flags < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    let inherited_flags = current_flags & !libc::FD_CLOEXEC;
                    if libc::fcntl(seccomp_fd, libc::F_SETFD, inherited_flags) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
    }

    pub fn prepare_proof_clock_seccomp_filter() -> Result<ProofClockSeccompFilter, String> {
        let instructions = proof_clock_filter_instructions()?;
        let identity = filter_identity(&instructions);
        let mut file =
            tempfile::tempfile().map_err(|err| format!("creating proof clock seccomp filter file: {err}"))?;
        let bytes = instruction_bytes(&instructions);
        file.write_all(bytes).map_err(|err| format!("writing proof clock seccomp filter: {err}"))?;
        file.seek(SeekFrom::Start(0))
            .map_err(|err| format!("rewinding proof clock seccomp filter: {err}"))?;
        assert!(!instructions.is_empty(), "proof clock seccomp filter must contain instructions");
        assert!(!identity.is_empty(), "proof clock seccomp filter identity must be present");
        Ok(ProofClockSeccompFilter { file, identity })
    }

    pub fn proof_clock_filter_identity() -> Result<String, String> {
        let instructions = proof_clock_filter_instructions()?;
        let identity = filter_identity(&instructions);
        assert!(!instructions.is_empty(), "proof clock seccomp filter must contain instructions");
        assert!(identity.starts_with(FILTER_IDENTITY_PREFIX));
        Ok(identity)
    }

    fn proof_clock_filter_instructions() -> Result<Vec<libc::sock_filter>, String> {
        let audit_arch = current_audit_arch()
            .ok_or_else(|| "proof clock seccomp filtering is unsupported on this Linux architecture".to_string())?;
        let denied_syscalls = denied_clock_syscall_numbers()?;
        if denied_syscalls.is_empty() {
            return Err("proof clock seccomp filtering has no denied syscalls".to_string());
        }
        let instruction_capacity_count = FILTER_FIXED_INSTRUCTION_COUNT
            .checked_add(
                denied_syscalls
                    .len()
                    .checked_mul(FILTER_INSTRUCTIONS_PER_DENIED_SYSCALL)
                    .ok_or_else(|| "proof clock seccomp instruction count overflowed".to_string())?,
            )
            .ok_or_else(|| "proof clock seccomp instruction count overflowed".to_string())?;
        let mut instructions = Vec::with_capacity(instruction_capacity_count);
        instructions.push(bpf_stmt(libc::BPF_LD | libc::BPF_W | libc::BPF_ABS, SECCOMP_DATA_ARCH_OFFSET_BYTES)?);
        instructions.push(bpf_jump(libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K, audit_arch, 1, 0)?);
        instructions.push(bpf_stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_KILL_PROCESS)?);
        instructions.push(bpf_stmt(libc::BPF_LD | libc::BPF_W | libc::BPF_ABS, SECCOMP_DATA_NR_OFFSET_BYTES)?);
        for syscall_number in denied_syscalls {
            instructions.push(bpf_jump(libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K, syscall_number, 0, 1)?);
            instructions.push(bpf_stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_ERRNO | FILTER_DENIAL_ERRNO)?);
        }
        instructions.push(bpf_stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_ALLOW)?);
        assert_eq!(instructions.len(), instruction_capacity_count);
        assert!(instructions.len() > FILTER_FIXED_INSTRUCTION_COUNT);
        Ok(instructions)
    }

    fn denied_clock_syscall_numbers() -> Result<Vec<u32>, String> {
        let mut syscalls = vec![
            checked_syscall_number(libc::SYS_clock_gettime, "clock_gettime")?,
            checked_syscall_number(libc::SYS_clock_getres, "clock_getres")?,
            checked_syscall_number(libc::SYS_clock_nanosleep, "clock_nanosleep")?,
            checked_syscall_number(libc::SYS_clock_settime, "clock_settime")?,
            checked_syscall_number(libc::SYS_gettimeofday, "gettimeofday")?,
            checked_syscall_number(libc::SYS_settimeofday, "settimeofday")?,
            checked_syscall_number(libc::SYS_adjtimex, "adjtimex")?,
            checked_syscall_number(libc::SYS_clock_adjtime, "clock_adjtime")?,
            checked_syscall_number(libc::SYS_nanosleep, "nanosleep")?,
            checked_syscall_number(libc::SYS_getitimer, "getitimer")?,
            checked_syscall_number(libc::SYS_setitimer, "setitimer")?,
            checked_syscall_number(libc::SYS_timer_create, "timer_create")?,
            checked_syscall_number(libc::SYS_timer_settime, "timer_settime")?,
            checked_syscall_number(libc::SYS_timer_gettime, "timer_gettime")?,
            checked_syscall_number(libc::SYS_timer_getoverrun, "timer_getoverrun")?,
            checked_syscall_number(libc::SYS_timer_delete, "timer_delete")?,
            checked_syscall_number(libc::SYS_timerfd_create, "timerfd_create")?,
            checked_syscall_number(libc::SYS_timerfd_settime, "timerfd_settime")?,
            checked_syscall_number(libc::SYS_timerfd_gettime, "timerfd_gettime")?,
        ];
        #[cfg(target_arch = "x86_64")]
        {
            syscalls.push(checked_syscall_number(libc::SYS_time, "time")?);
            syscalls.push(checked_syscall_number(libc::SYS_alarm, "alarm")?);
        }
        syscalls.sort_unstable();
        syscalls.dedup();
        assert!(!syscalls.is_empty(), "proof clock seccomp syscall set must not be empty");
        assert!(syscalls.windows(2).all(|pair| pair[0] < pair[1]));
        Ok(syscalls)
    }

    fn checked_syscall_number(number: libc::c_long, name: &str) -> Result<u32, String> {
        if name.is_empty() {
            return Err("proof clock seccomp syscall name must not be empty".to_string());
        }
        let number = u32::try_from(number)
            .map_err(|_| format!("proof clock seccomp syscall {name} has unsupported number {number}"))?;
        assert!(!name.is_empty());
        assert!(number > 0, "proof clock seccomp syscall number must be positive");
        Ok(number)
    }

    fn filter_identity(instructions: &[libc::sock_filter]) -> String {
        assert!(!instructions.is_empty(), "proof clock seccomp filter must contain instructions");
        let digest = blake3::hash(instruction_bytes(instructions)).to_hex().to_string();
        assert!(!digest.is_empty(), "proof clock seccomp filter digest must be present");
        assert_eq!(digest.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
        format!("{FILTER_IDENTITY_PREFIX}:{digest}")
    }

    fn instruction_bytes(instructions: &[libc::sock_filter]) -> &[u8] {
        let byte_len = std::mem::size_of_val(instructions);
        assert!(!instructions.is_empty(), "proof clock seccomp filter must contain instructions");
        assert!(byte_len >= std::mem::size_of::<libc::sock_filter>());
        // SAFETY: `sock_filter` is a plain C data structure, and this view is bounded to the initialized
        // slice.
        unsafe { std::slice::from_raw_parts(instructions.as_ptr().cast::<u8>(), byte_len) }
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

    fn bpf_stmt(code: u32, k: u32) -> Result<libc::sock_filter, String> {
        let encoded_code = u16::try_from(code).map_err(|_| format!("BPF statement code exceeds u16: {code}"))?;
        assert_eq!(u32::from(encoded_code), code);
        assert_ne!(code, 0, "BPF statement code must identify an operation");
        Ok(libc::sock_filter {
            code: encoded_code,
            jt: 0,
            jf: 0,
            k,
        })
    }

    fn bpf_jump(code: u32, k: u32, jump_true: u8, jump_false: u8) -> Result<libc::sock_filter, String> {
        let encoded_code = u16::try_from(code).map_err(|_| format!("BPF jump code exceeds u16: {code}"))?;
        assert_eq!(u32::from(encoded_code), code);
        assert_ne!(code, 0, "BPF jump code must identify an operation");
        Ok(libc::sock_filter {
            code: encoded_code,
            jt: jump_true,
            jf: jump_false,
            k,
        })
    }

    #[cfg(test)]
    mod tests {
        use std::process::Command;

        use super::*;

        const CLOCK_FILTER_PROBE_ENV: &str = "MANTLE_TEST_CLOCK_FILTER_PROBE";
        const CLOCK_FILTER_PROBE_TEST: &str =
            "proof_clock_seccomp::linux::tests::kernel_filter_denies_clock_syscall_child";
        const REAL_BWRAP_ENV: &str = "MANTLE_TEST_REAL_BWRAP";
        const BWRAP_FILTER_PROBE_ENV: &str = "MANTLE_TEST_BWRAP_CLOCK_FILTER_PROBE";
        const BWRAP_FILTER_PROBE_TEST: &str =
            "proof_clock_seccomp::linux::tests::bwrap_filter_denies_clock_syscall_child";
        const EXPECTED_CLOCK_GETTIME_ARGUMENT_COUNT: usize = 2;

        #[test]
        fn filter_contains_errno_denials_and_a_terminal_allow() {
            let instructions = proof_clock_filter_instructions().unwrap();
            let denial_action = libc::SECCOMP_RET_ERRNO | FILTER_DENIAL_ERRNO;
            let denial_count = instructions.iter().filter(|instruction| instruction.k == denial_action).count();
            let denied_syscall_count = denied_clock_syscall_numbers().unwrap().len();

            assert_eq!(denial_count, denied_syscall_count);
            assert_eq!(instructions.last().unwrap().k, libc::SECCOMP_RET_ALLOW);
            assert_ne!(instructions.last().unwrap().k, denial_action);
        }

        #[test]
        fn kernel_filter_denies_clock_syscall_and_allows_unrelated_syscall() {
            let output = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg(CLOCK_FILTER_PROBE_TEST)
                .arg("--nocapture")
                .env(CLOCK_FILTER_PROBE_ENV, "1")
                .output()
                .unwrap();

            assert!(output.status.success(), "child stderr: {}", String::from_utf8_lossy(&output.stderr));
            assert!(output.stdout.is_empty() || String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
        }

        #[test]
        fn kernel_filter_denies_clock_syscall_child() {
            if std::env::var_os(CLOCK_FILTER_PROBE_ENV).is_none() {
                return;
            }
            let instructions = proof_clock_filter_instructions().unwrap();
            install_filter_for_current_thread(&instructions).unwrap();
            assert_clock_syscall_denied_and_unrelated_syscall_allowed();
            // The filter cannot be removed from this process. Exit before the libtest harness performs
            // unrelated work.
            unsafe { libc::_exit(0) }
        }

        #[test]
        fn real_bwrap_applies_inherited_clock_filter() {
            let Some(bwrap) = std::env::var_os(REAL_BWRAP_ENV) else {
                return;
            };
            let current_exe = std::env::current_exe().unwrap();
            let filter = prepare_proof_clock_seccomp_filter().unwrap();
            let mut command = Command::new(bwrap);
            command
                .arg("--die-with-parent")
                .arg("--ro-bind")
                .arg("/")
                .arg("/")
                .arg("--seccomp")
                .arg(filter.fd().to_string())
                .arg(&current_exe)
                .arg("--exact")
                .arg(BWRAP_FILTER_PROBE_TEST)
                .arg("--nocapture")
                .env(BWRAP_FILTER_PROBE_ENV, "1");
            filter.configure_inheritance(&mut command);
            let output = command.output().unwrap();

            assert!(output.status.success(), "bwrap stderr: {}", String::from_utf8_lossy(&output.stderr));
            assert!(output.stdout.is_empty() || String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
        }

        #[test]
        fn bwrap_filter_denies_clock_syscall_child() {
            if std::env::var_os(BWRAP_FILTER_PROBE_ENV).is_none() {
                return;
            }
            assert_clock_syscall_denied_and_unrelated_syscall_allowed();
            unsafe { libc::_exit(0) }
        }

        fn assert_clock_syscall_denied_and_unrelated_syscall_allowed() {
            let mut clock_value: libc::timespec = unsafe { std::mem::zeroed() };
            let clock_rc = unsafe {
                libc::syscall(libc::SYS_clock_gettime, libc::CLOCK_REALTIME, &mut clock_value as *mut libc::timespec)
            };
            let clock_error = std::io::Error::last_os_error().raw_os_error();
            let getpid_rc = unsafe { libc::syscall(libc::SYS_getpid) };

            assert_eq!(EXPECTED_CLOCK_GETTIME_ARGUMENT_COUNT, 2);
            assert_eq!(clock_rc, -1);
            assert_eq!(clock_error, Some(libc::EPERM));
            assert!(getpid_rc > 0);
        }

        fn install_filter_for_current_thread(instructions: &[libc::sock_filter]) -> Result<(), String> {
            let instruction_count = u16::try_from(instructions.len())
                .map_err(|_| "proof clock seccomp test filter is too large".to_string())?;
            let mut program = libc::sock_fprog {
                len: instruction_count,
                filter: instructions.as_ptr().cast_mut(),
            };
            let no_new_privileges_rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
            if no_new_privileges_rc != 0 {
                return Err(format!("setting no_new_privileges: {}", std::io::Error::last_os_error()));
            }
            let seccomp_rc = unsafe {
                libc::syscall(
                    libc::SYS_seccomp,
                    libc::SECCOMP_SET_MODE_FILTER,
                    0,
                    &mut program as *mut libc::sock_fprog,
                )
            };
            if seccomp_rc != 0 {
                return Err(format!("installing proof clock seccomp test filter: {}", std::io::Error::last_os_error()));
            }
            assert_eq!(program.len, instruction_count);
            assert!(!program.filter.is_null());
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod linux {
    #[derive(Debug)]
    pub struct ProofClockSeccompFilter;

    impl ProofClockSeccompFilter {
        pub fn fd(&self) -> libc::c_int {
            unreachable!("proof clock seccomp filtering requires Linux")
        }

        pub fn identity(&self) -> &str {
            unreachable!("proof clock seccomp filtering requires Linux")
        }

        pub fn configure_inheritance(&self, _command: &mut std::process::Command) {
            unreachable!("proof clock seccomp filtering requires Linux")
        }
    }

    pub fn prepare_proof_clock_seccomp_filter() -> Result<ProofClockSeccompFilter, String> {
        Err("proof clock seccomp filtering requires Linux".to_string())
    }

    pub fn proof_clock_filter_identity() -> Result<String, String> {
        Err("proof clock seccomp filtering requires Linux".to_string())
    }
}

pub use linux::ProofClockSeccompFilter;
pub use linux::prepare_proof_clock_seccomp_filter;
pub use linux::proof_clock_filter_identity;
