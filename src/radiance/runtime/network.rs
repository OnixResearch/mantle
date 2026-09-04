#![cfg(target_os = "linux")]

use std::os::unix::process::CommandExt;

const BPF_LOAD_ABSOLUTE_WORD: u16 = 0x20;
const BPF_JUMP_EQUAL: u16 = 0x15;
const BPF_RETURN: u16 = 0x06;
const SECCOMP_DATA_NUMBER_OFFSET: u32 = 0;
const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
const SECCOMP_DATA_FIRST_ARGUMENT_OFFSET: u32 = 16;
const SECCOMP_RETURN_KILL_PROCESS: u32 = 0x8000_0000;
const SECCOMP_RETURN_TRAP: u32 = 0x0003_0000;
const SECCOMP_RETURN_ALLOW: u32 = 0x7fff_0000;
#[cfg(target_arch = "x86_64")]
const AUDIT_ARCH_X86_64: u32 = 0xc000_003e;
#[cfg(target_arch = "aarch64")]
const AUDIT_ARCH_AARCH64: u32 = 0xc000_00b7;
const PRCTL_SUCCESS: libc::c_int = 0;
const FILTER_NEXT_INSTRUCTION: u8 = 0;
const FILTER_SKIP_ONE: u8 = 1;
const FILTER_SKIP_TWO: u8 = 2;
const FILTER_INSTRUCTION_COUNT: usize = 13;
const FIRST_NON_STANDARD_FD: u32 = 3;
const CLOSE_RANGE_CLOEXEC: u32 = 4;

#[derive(Clone, Copy)]
struct JumpOffsets {
    if_equal: u8,
    if_not_equal: u8,
}

const EQUAL_SKIP_ONE: JumpOffsets = JumpOffsets {
    if_equal: FILTER_SKIP_ONE,
    if_not_equal: FILTER_NEXT_INSTRUCTION,
};
const NOT_EQUAL_SKIP_ONE: JumpOffsets = JumpOffsets {
    if_equal: FILTER_NEXT_INSTRUCTION,
    if_not_equal: FILTER_SKIP_ONE,
};
const EQUAL_SKIP_TWO: JumpOffsets = JumpOffsets {
    if_equal: FILTER_SKIP_TWO,
    if_not_equal: FILTER_NEXT_INSTRUCTION,
};

pub(super) fn install_pre_exec(command: &mut std::process::Command) -> std::io::Result<()> {
    let architecture = current_audit_arch()?;
    let mut filter = filter_program(architecture)?;
    // SAFETY: the bounded filter is allocated before std::process::Command::spawn. The closure calls
    // only libc operations after fork. It closes inherited network authority at exec and traps
    // creation of non-local sockets or io_uring. A successful stage therefore made no network
    // request.
    unsafe {
        command.pre_exec(move || {
            mark_inherited_descriptors_close_on_exec()?;
            install_program(&mut filter)
        });
    }
    Ok(())
}

fn filter_program(architecture: u32) -> std::io::Result<Vec<libc::sock_filter>> {
    let io_uring_setup_syscall = u32::try_from(libc::SYS_io_uring_setup)
        .map_err(|_| std::io::Error::other("io_uring_setup syscall number does not fit u32"))?;
    let socket_syscall =
        u32::try_from(libc::SYS_socket).map_err(|_| std::io::Error::other("socket syscall number does not fit u32"))?;
    let socketpair_syscall = u32::try_from(libc::SYS_socketpair)
        .map_err(|_| std::io::Error::other("socketpair syscall number does not fit u32"))?;
    let local_domain =
        u32::try_from(libc::AF_UNIX).map_err(|_| std::io::Error::other("AF_UNIX domain does not fit u32"))?;
    let filter = vec![
        statement(BPF_LOAD_ABSOLUTE_WORD, SECCOMP_DATA_ARCH_OFFSET),
        jump(BPF_JUMP_EQUAL, architecture, EQUAL_SKIP_ONE),
        statement(BPF_RETURN, SECCOMP_RETURN_KILL_PROCESS),
        statement(BPF_LOAD_ABSOLUTE_WORD, SECCOMP_DATA_NUMBER_OFFSET),
        jump(BPF_JUMP_EQUAL, io_uring_setup_syscall, NOT_EQUAL_SKIP_ONE),
        statement(BPF_RETURN, SECCOMP_RETURN_TRAP),
        jump(BPF_JUMP_EQUAL, socket_syscall, EQUAL_SKIP_TWO),
        jump(BPF_JUMP_EQUAL, socketpair_syscall, EQUAL_SKIP_ONE),
        statement(BPF_RETURN, SECCOMP_RETURN_ALLOW),
        statement(BPF_LOAD_ABSOLUTE_WORD, SECCOMP_DATA_FIRST_ARGUMENT_OFFSET),
        jump(BPF_JUMP_EQUAL, local_domain, EQUAL_SKIP_ONE),
        statement(BPF_RETURN, SECCOMP_RETURN_TRAP),
        statement(BPF_RETURN, SECCOMP_RETURN_ALLOW),
    ];
    debug_assert_eq!(filter.len(), FILTER_INSTRUCTION_COUNT);
    debug_assert!(!filter.is_empty());
    Ok(filter)
}

fn mark_inherited_descriptors_close_on_exec() -> std::io::Result<()> {
    let result = unsafe { libc::syscall(libc::SYS_close_range, FIRST_NON_STANDARD_FD, u32::MAX, CLOSE_RANGE_CLOEXEC) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    debug_assert!(FIRST_NON_STANDARD_FD > libc::STDERR_FILENO as u32);
    debug_assert_ne!(CLOSE_RANGE_CLOEXEC, 0);
    Ok(())
}

fn install_program(filter: &mut [libc::sock_filter]) -> std::io::Result<()> {
    let instruction_count =
        u16::try_from(filter.len()).map_err(|_| std::io::Error::other("network filter length exceeds u16"))?;
    let mut program = libc::sock_fprog {
        len: instruction_count,
        filter: filter.as_mut_ptr(),
    };
    let no_new_privileges = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1_u64, 0_u64, 0_u64, 0_u64) };
    if no_new_privileges != PRCTL_SUCCESS {
        return Err(std::io::Error::last_os_error());
    }
    let installed =
        unsafe { libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &mut program as *mut libc::sock_fprog) };
    if installed != PRCTL_SUCCESS {
        return Err(std::io::Error::last_os_error());
    }
    debug_assert_eq!(usize::from(instruction_count), filter.len());
    debug_assert!(!filter.is_empty());
    Ok(())
}

fn current_audit_arch() -> std::io::Result<u32> {
    #[cfg(target_arch = "x86_64")]
    {
        Ok(AUDIT_ARCH_X86_64)
    }
    #[cfg(target_arch = "aarch64")]
    {
        Ok(AUDIT_ARCH_AARCH64)
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        Err(std::io::Error::other("Radiance network filter supports only x86_64 and aarch64 Linux"))
    }
}

const fn statement(code: u16, value: u32) -> libc::sock_filter {
    libc::sock_filter {
        code,
        jt: FILTER_NEXT_INSTRUCTION,
        jf: FILTER_NEXT_INSTRUCTION,
        k: value,
    }
}

const fn jump(code: u16, value: u32, offsets: JumpOffsets) -> libc::sock_filter {
    libc::sock_filter {
        code,
        jt: offsets.if_equal,
        jf: offsets.if_not_equal,
        k: value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARCH_LOAD_INDEX: usize = 0;
    const ARCH_COMPARE_INDEX: usize = 1;
    const ARCH_KILL_INDEX: usize = 2;
    const IO_URING_COMPARE_INDEX: usize = 4;
    const IO_URING_TRAP_INDEX: usize = 5;
    const SOCKET_COMPARE_INDEX: usize = 6;
    const SOCKETPAIR_COMPARE_INDEX: usize = 7;
    const DOMAIN_LOAD_INDEX: usize = 9;
    const LOCAL_DOMAIN_COMPARE_INDEX: usize = 10;
    const NON_LOCAL_TRAP_INDEX: usize = 11;

    #[test]
    fn program_allows_only_local_socket_creation() {
        let architecture = current_audit_arch().unwrap();
        let filter = filter_program(architecture).unwrap();
        assert_eq!(filter[SOCKET_COMPARE_INDEX].k, u32::try_from(libc::SYS_socket).unwrap());
        assert_eq!(filter[SOCKETPAIR_COMPARE_INDEX].k, u32::try_from(libc::SYS_socketpair).unwrap());
        assert_eq!(filter[DOMAIN_LOAD_INDEX].k, SECCOMP_DATA_FIRST_ARGUMENT_OFFSET);
        assert_eq!(filter[LOCAL_DOMAIN_COMPARE_INDEX].k, u32::try_from(libc::AF_UNIX).unwrap());
        assert_eq!(filter[NON_LOCAL_TRAP_INDEX].k, SECCOMP_RETURN_TRAP);
        assert_eq!(filter.last().map(|instruction| instruction.k), Some(SECCOMP_RETURN_ALLOW));
    }

    #[test]
    fn program_kills_wrong_arch_and_traps_io_uring() {
        let architecture = current_audit_arch().unwrap();
        let filter = filter_program(architecture).unwrap();
        assert_eq!(filter[ARCH_LOAD_INDEX].k, SECCOMP_DATA_ARCH_OFFSET);
        assert_eq!(filter[ARCH_COMPARE_INDEX].k, architecture);
        assert_eq!(filter[ARCH_KILL_INDEX].k, SECCOMP_RETURN_KILL_PROCESS);
        assert_eq!(filter[IO_URING_COMPARE_INDEX].k, u32::try_from(libc::SYS_io_uring_setup).unwrap());
        assert_eq!(filter[IO_URING_TRAP_INDEX].k, SECCOMP_RETURN_TRAP);
    }
}
