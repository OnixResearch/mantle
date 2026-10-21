//! Allocation-free descriptor RPC to the sole seccomp notification listener.
//! SCM_RIGHTS and ADDFD retain OFDs, including aliases and actual final offsets.
use super::*;
#[repr(C)]
struct ResponsePacket {
    response: libc::seccomp_notif_resp,
    descriptors_count: u32,
    close_on_exec: [u32; 64],
}
#[repr(C)]
struct AddFd {
    id: u64,
    flags: u32,
    source_fd: u32,
    target_fd: u32,
    target_flags: u32,
}

pub(super) fn send(
    socket: i32,
    response: &libc::seccomp_notif_resp,
    injections: Option<&[command_state::DescriptorInjection; 64]>,
) -> Result<(), String> {
    let mut packet: ResponsePacket = unsafe { std::mem::zeroed() };
    packet.response = libc::seccomp_notif_resp {
        id: response.id,
        val: response.val,
        error: response.error,
        flags: response.flags,
    };
    let mut source_fds = [-1i32; 64];
    if let Some(injections) = injections {
        packet.descriptors_count = 64;
        for (index, injection) in injections.iter().enumerate() {
            if injection.target_fd != index as u32 || injection.source_fd < 0 {
                return Err("invalid exact raw descriptor injection".into());
            }
            source_fds[index] = injection.source_fd;
            packet.close_on_exec[index] = u32::from(injection.close_on_exec);
        }
    }
    let mut iovec = libc::iovec {
        iov_base: (&mut packet as *mut ResponsePacket).cast(),
        iov_len: std::mem::size_of::<ResponsePacket>(),
    };
    let mut control = [0usize; 40];
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &mut iovec;
    message.msg_iovlen = 1;
    if injections.is_some() {
        message.msg_control = control.as_mut_ptr().cast();
        message.msg_controllen = unsafe { libc::CMSG_SPACE(std::mem::size_of_val(&source_fds) as u32) } as usize;
        unsafe {
            let header = libc::CMSG_FIRSTHDR(&message);
            (*header).cmsg_level = libc::SOL_SOCKET;
            (*header).cmsg_type = libc::SCM_RIGHTS;
            (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of_val(&source_fds) as u32) as usize;
            std::ptr::copy_nonoverlapping(source_fds.as_ptr(), libc::CMSG_DATA(header).cast::<i32>(), 64);
        }
    }
    if unsafe { libc::sendmsg(socket, &message, libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL) }
        != std::mem::size_of::<ResponsePacket>() as isize
    {
        return Err("sending protected descriptor decision failed".into());
    }
    Ok(())
}

/// Runs only in the post-fork syscall-only pump. No allocator, locks, destructors,
/// ambient file opens, or executable wrapper participates in raw inheritance.
pub(super) unsafe fn receive_apply(socket: i32, listener: i32, id: u64) -> i32 {
    unsafe {
        let mut packet: ResponsePacket = std::mem::zeroed();
        let mut control = [0usize; 40];
        let mut iovec = libc::iovec {
            iov_base: (&mut packet as *mut ResponsePacket).cast(),
            iov_len: std::mem::size_of::<ResponsePacket>(),
        };
        let mut message: libc::msghdr = std::mem::zeroed();
        message.msg_iov = &mut iovec;
        message.msg_iovlen = 1;
        message.msg_control = control.as_mut_ptr().cast();
        message.msg_controllen = std::mem::size_of_val(&control);
        if libc::syscall(libc::SYS_recvmsg, socket, &mut message, libc::MSG_CMSG_CLOEXEC | libc::MSG_TRUNC)
            != std::mem::size_of::<ResponsePacket>() as libc::c_long
            || message.msg_flags & (libc::MSG_CTRUNC | libc::MSG_TRUNC) != 0
            || packet.response.id != id
        {
            return libc::EPROTO;
        }
        let header = libc::CMSG_FIRSTHDR(&message);
        let mut failure = 0;
        if packet.descriptors_count == 64 {
            if header.is_null()
                || (*header).cmsg_level != libc::SOL_SOCKET
                || (*header).cmsg_type != libc::SCM_RIGHTS
                || (*header).cmsg_len != libc::CMSG_LEN((64 * std::mem::size_of::<i32>()) as u32) as usize
            {
                return libc::EPROTO;
            }
            let source_fds = libc::CMSG_DATA(header).cast::<i32>();
            for index in 0..64usize {
                let source_fd = std::ptr::read_unaligned(source_fds.add(index));
                if failure == 0 {
                    if packet.close_on_exec[index] > 1 {
                        failure = libc::EPROTO;
                    } else {
                        let request = AddFd {
                            id,
                            flags: 1,
                            source_fd: source_fd as u32,
                            target_fd: index as u32,
                            target_flags: if packet.close_on_exec[index] == 1 {
                                libc::O_CLOEXEC as u32
                            } else {
                                0
                            },
                        };
                        let result = libc::syscall(libc::SYS_ioctl, listener, 0x4018_2103u64, &request);
                        if result != index as libc::c_long {
                            failure = if result < 0 {
                                *libc::__errno_location()
                            } else {
                                libc::EPROTO
                            };
                        }
                    }
                }
                libc::syscall(libc::SYS_close, source_fd);
            }
        } else if packet.descriptors_count != 0 || !header.is_null() {
            return libc::EPROTO;
        }
        if failure != 0 {
            packet.response.error = -libc::EPERM;
            packet.response.flags = 0;
        }
        if libc::syscall(libc::SYS_ioctl, listener, 0xC018_2101u64, &packet.response) != 0 && failure == 0 {
            failure = *libc::__errno_location();
        }
        failure
    }
}
