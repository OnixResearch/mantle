//! Linux ptrace-based exec supervision (ADR 0086).
//!
//! Replaces `SECCOMP_RET_USER_NOTIF` continue-on-execve, which empirically
//! returned spurious `EACCES` to bound executables under sustained parallel
//! load. Here the kernel stops each traced task at `execve`/`execveat`
//! syscall entry (`SECCOMP_RET_TRACE` + `PTRACE_EVENT_SECCOMP`); the tracer
//! classifies with the same exact-byte policy and either resumes the exec or
//! rewrites registers so the syscall returns `EACCES` without executing.
#![cfg(target_os = "linux")]

mod linux {
    use std::collections::BTreeSet;
    use std::io;
    use std::io::Read;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::fd::IntoRawFd;
    use std::os::unix::process::CommandExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::RwLock;
    use std::thread;

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
    const SECCOMP_DATA_NR_OFFSET: u32 = 0;
    const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
    const SECCOMP_RET_TRACE: u32 = 0x7FF0_0000;
    const SECCOMP_RET_ALLOW: u32 = 0x7FFF_0000;
    const SECCOMP_SET_MODE_FILTER: libc::c_long = 1;
    const SYS_SECCOMP: libc::c_long = 317;
    const PTRACE_TRACEME: libc::c_uint = 0;
    const PTRACE_SETOPTIONS: libc::c_uint = 0x4200;
    const PTRACE_SEIZE: libc::c_uint = 0x4206;
    const PTRACE_CONT: libc::c_uint = 7;
    const PTRACE_GETREGS: libc::c_uint = 12;
    const PTRACE_SETREGS: libc::c_uint = 13;
    const PTRACE_O_TRACESYSGOOD: libc::c_uint = 1;
    const PTRACE_O_TRACEFORK: libc::c_uint = 2;
    const PTRACE_O_TRACEVFORK: libc::c_uint = 4;
    const PTRACE_O_TRACECLONE: libc::c_uint = 8;
    const PTRACE_O_TRACESECCOMP: libc::c_uint = 0x80;
    const PTRACE_EVENT_SECCOMP: u32 = 7;
    const STOP_LOW_BYTE_MARKER: u32 = 0x7F;
    const SIGTRAP_BYTE: u32 = libc::SIGTRAP as u32;
    const SYSCALL_STOP_SIGTRAP_BYTE: u32 = libc::SIGTRAP as u32 | 0x80;
    const EXECVE_NR: u64 = 59;
    const EXECVEAT_NR: u64 = 322;
    const AT_FDCWD: i64 = -100;
    const DENIED_SYSCALL_MARKER: u64 = u64::MAX;
    const DENIED_RETURN: u64 = (-(libc::EACCES as i64)) as u64;

    #[derive(Debug, Eq, PartialEq)]
    pub enum PtraceSupervisorError {
        Unsupported(String),
        Install(String),
        Attach(String),
        TracerLoop(String),
    }

    impl std::fmt::Display for PtraceSupervisorError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unsupported(m) => write!(f, "ptrace supervisor unsupported: {m}"),
                Self::Install(m) => write!(f, "installing ptrace supervision failed: {m}"),
                Self::Attach(m) => write!(f, "attaching ptrace supervisor failed: {m}"),
                Self::TracerLoop(m) => write!(f, "ptrace tracer loop failed: {m}"),
            }
        }
    }

    impl std::error::Error for PtraceSupervisorError {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ExecStop {
        Execve { pid: i32 },
        Execveat { pid: i32 },
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct EntryRegs {
        syscall_nr: u64,
        arg0: u64,
        arg1: u64,
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

    fn last_error(context: &str) -> String {
        format!("{context}: {}", io::Error::last_os_error())
    }

    fn exec_trace_filter(arch: u32) -> Result<[libc::sock_filter; FILTER_INSTRUCTION_COUNT], PtraceSupervisorError> {
        assert_ne!(arch, 0);
        let kill_process = libc::SECCOMP_RET_KILL_PROCESS;
        Ok([
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
                operand: kill_process,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                operand: SECCOMP_DATA_NR_OFFSET,
            }),
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: EXECVE_NR as u32,
                jump_true: 0,
                jump_false: 0,
            }),
            bpf_jump(BpfJump {
                code: libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                operand: EXECVEAT_NR as u32,
                jump_true: 0,
                jump_false: 0,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: SECCOMP_RET_TRACE,
            }),
            bpf_stmt(BpfStatement {
                code: libc::BPF_RET | libc::BPF_K,
                operand: SECCOMP_RET_ALLOW,
            }),
        ])
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
        arch.ok_or_else(|| {
            PtraceSupervisorError::Unsupported("unsupported Linux audit architecture for exec supervisor".to_string())
        })
    }

    fn pre_exec_install_trace_filter() -> Result<(), PtraceSupervisorError> {
        let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1u64, 0u64, 0u64, 0u64) };
        if rc != 0 {
            return Err(PtraceSupervisorError::Install(last_error("PR_SET_NO_NEW_PRIVS")));
        }
        let arch = require_supported_audit_arch(current_audit_arch())?;
        let mut filter = exec_trace_filter(arch)?;
        let filter_len = u16::try_from(filter.len())
            .map_err(|_| PtraceSupervisorError::Install("filter length exceeds u16".to_string()))?;
        let mut program = libc::sock_fprog {
            len: filter_len,
            filter: filter.as_mut_ptr(),
        };
        assert_eq!(filter.len(), FILTER_INSTRUCTION_COUNT);
        let rc = unsafe {
            libc::syscall(
                SYS_SECCOMP,
                SECCOMP_SET_MODE_FILTER,
                0 as libc::c_ulong,
                &mut program as *mut libc::sock_fprog,
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::Install(last_error("SECCOMP_SET_MODE_FILTER")));
        }
        let rc = unsafe {
            libc::ptrace(
                PTRACE_TRACEME,
                0 as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                std::ptr::null_mut::<libc::c_void>(),
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::Install(last_error("PTRACE_TRACEME")));
        }
        Ok(())
    }

    fn pre_exec_raise_stop() {
        unsafe {
            libc::raise(libc::SIGSTOP);
        }
    }

    fn seize(pid: i32) -> Result<(), PtraceSupervisorError> {
        let rc = unsafe {
            libc::ptrace(
                PTRACE_SEIZE,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                std::ptr::null_mut::<libc::c_void>(),
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::Attach(last_error("PTRACE_SEIZE")));
        }
        Ok(())
    }

    fn set_options(pid: i32, options: libc::c_uint) -> Result<(), PtraceSupervisorError> {
        let rc = unsafe {
            libc::ptrace(
                PTRACE_SETOPTIONS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                options as libc::c_ulonglong as *mut libc::c_void,
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::Attach(last_error("PTRACE_SETOPTIONS")));
        }
        Ok(())
    }

    fn cont(pid: i32) -> Result<(), PtraceSupervisorError> {
        let rc = unsafe {
            libc::ptrace(
                PTRACE_CONT,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                std::ptr::null_mut::<libc::c_void>(),
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_CONT")));
        }
        Ok(())
    }

    fn get_entry_regs(pid: i32) -> Result<EntryRegs, PtraceSupervisorError> {
        let mut regs: libc::user_regs_struct = unsafe { std::mem::zeroed() };
        let rc = unsafe {
            libc::ptrace(
                PTRACE_GETREGS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &mut regs as *mut libc::user_regs_struct as *mut libc::c_void,
            )
        };
        if rc != 0 {
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
        let rc = unsafe {
            libc::ptrace(
                PTRACE_GETREGS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &mut regs as *mut libc::user_regs_struct as *mut libc::c_void,
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_GETREGS deny")));
        }
        regs.rax = DENIED_RETURN;
        regs.orig_rax = DENIED_SYSCALL_MARKER;
        let rc = unsafe {
            libc::ptrace(
                PTRACE_SETREGS,
                pid as libc::pid_t,
                std::ptr::null_mut::<libc::c_void>(),
                &regs as *const libc::user_regs_struct as *const libc::c_void,
            )
        };
        if rc != 0 {
            return Err(PtraceSupervisorError::TracerLoop(last_error("PTRACE_SETREGS deny")));
        }
        Ok(())
    }

    fn is_seccomp_event_stop(wait_status: i32) -> bool {
        let bits = wait_status as u32;
        if bits & 0xFF != STOP_LOW_BYTE_MARKER {
            return false;
        }
        let stop_signal = (bits >> 8) & 0xFF;
        if stop_signal != SIGTRAP_BYTE {
            return false;
        }
        (bits >> 16) & 0xFF == PTRACE_EVENT_SECCOMP
    }

    fn is_syscall_stop(wait_status: i32) -> bool {
        let bits = wait_status as u32;
        bits & 0xFF == STOP_LOW_BYTE_MARKER && (bits >> 8) & 0xFF == SYSCALL_STOP_SIGTRAP_BYTE
    }

    fn is_signal_delivery_stop(wait_status: i32) -> bool {
        libc::WIFSTOPPED(wait_status)
            && !is_seccomp_event_stop(wait_status)
            && !is_syscall_stop(wait_status)
            && libc::WSTOPSIG(wait_status) != libc::SIGSTOP
    }

    fn is_initial_group_stop(wait_status: i32) -> bool {
        libc::WIFSTOPPED(wait_status) && libc::WSTOPSIG(wait_status) == libc::SIGSTOP
    }

    fn exec_entry_from_stop(wait_status: i32, pid: i32) -> Result<ExecStop, PtraceSupervisorError> {
        assert!(is_seccomp_event_stop(wait_status));
        let regs = get_entry_regs(pid)?;
        if regs.syscall_nr == EXECVE_NR && regs.arg0 != 0 {
            return Ok(ExecStop::Execve { pid });
        }
        if regs.syscall_nr == EXECVEAT_NR {
            let dirfd = i64::from_ne_bytes(regs.arg0.to_ne_bytes());
            if dirfd == AT_FDCWD && regs.arg1 != 0 {
                return Ok(ExecStop::Execveat { pid });
            }
            return Err(PtraceSupervisorError::TracerLoop(format!(
                "unsupported execveat dirfd {dirfd:#x}; only AT_FDCWD is supported"
            )));
        }
        Err(PtraceSupervisorError::TracerLoop(format!(
            "seccomp event stop for unsupported syscall {:#x}",
            regs.syscall_nr
        )))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const TEST_ARCH_X86_64: u32 = 0xC000_003E;
        const TEST_ARCH_ARM64: u32 = 0xC000_00B7;
        const TEST_FILTER_TRACE_INDEX: usize = 6;
        const TEST_FILTER_ALLOW_TAIL_INDEX: usize = 7;
        const TEST_EXECVE_NR: u32 = 59;
        const TEST_EXECVEAT_NR: u32 = 322;

        #[test]
        fn unsupported_architecture_is_rejected_before_filtering() {
            let error = require_supported_audit_arch(None).unwrap_err();
            assert_eq!(
                error.to_string(),
                "ptrace supervisor unsupported: unsupported Linux audit architecture for exec supervisor"
            );
        }

        #[test]
        fn trace_filter_traces_only_exec_syscalls_on_supported_arch() {
            let filter = exec_trace_filter(TEST_ARCH_X86_64).unwrap();
            assert_eq!(filter[0].k, SECCOMP_DATA_ARCH_OFFSET);
            assert_eq!(filter[2].k, libc::SECCOMP_RET_KILL_PROCESS);
            assert_eq!(filter[4].k, TEST_EXECVE_NR);
            assert_eq!(filter[5].k, TEST_EXECVEAT_NR);
            assert_eq!(filter[TEST_FILTER_TRACE_INDEX].k, SECCOMP_RET_TRACE);
            assert_eq!(filter[TEST_FILTER_ALLOW_TAIL_INDEX].k, SECCOMP_RET_ALLOW);
            let arm_filter = exec_trace_filter(TEST_ARCH_ARM64).unwrap();
            assert_eq!(arm_filter[1].k, TEST_ARCH_ARM64);
            assert_eq!(arm_filter[TEST_FILTER_TRACE_INDEX].k, SECCOMP_RET_TRACE);
        }

        #[test]
        fn seccomp_event_stops_are_recognized_and_syscall_stops_are_not() {
            let event_status = (PTRACE_EVENT_SECCOMP << 16) | (SIGTRAP_BYTE << 8) | STOP_LOW_BYTE_MARKER;
            assert!(is_seccomp_event_stop(event_status as i32));
            assert!(!is_syscall_stop(event_status as i32));
            let syscall_status = (SYSCALL_STOP_SIGTRAP_BYTE << 8) | STOP_LOW_BYTE_MARKER;
            assert!(is_syscall_stop(syscall_status as i32));
            assert!(!is_seccomp_event_stop(syscall_status as i32));
        }
    }
}
