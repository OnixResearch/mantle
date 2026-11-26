//! Linux cgroup-v2 hard resource enforcement and capability probing for
//! local Bubblewrap sandboxes.

use std::collections::BTreeSet;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::fs::{self};
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use nix::libc;
use tokio::process::Command;

use crate::buildservice::SandboxEffectiveLimit;
use crate::buildservice::SandboxEnforcementMode;
use crate::buildservice::SandboxHelperScopeOutcome;
use crate::buildservice::SandboxHelperScopePolicy;
use crate::buildservice::SandboxLiveCpuUnavailableReason;
use crate::buildservice::SandboxResourceEnforcementFacts;
use crate::buildservice::SandboxResourcePolicy;
use crate::buildservice::SandboxSwapPolicy;
use crate::buildservice::SandboxUnavailableReason;

const CGROUP2_SUPER_MAGIC: u64 = 0x6367_7270;
const MAX_CGROUP_READ_BYTES: usize = 4096;
const MAX_MOUNTINFO_READ_BYTES: usize = 65_536;
const MAX_PROC_SELF_FD_SCAN: usize = 65_536;
const MAX_PROC_NET_UNIX_BYTES: usize = 8 * 1024 * 1024;
const MAX_PROC_NET_UNIX_LINES: usize = 262_144;
const SOCK_DIAG_BY_FAMILY: u16 = 20;
const UDIAG_SHOW_VFS: u32 = 0x0000_0002;
const UNIX_DIAG_VFS: u16 = 1;
const NLMSG_ERROR_TYPE: u16 = 2;
const NLMSG_DONE_TYPE: u16 = 3;
const MAX_HELPER_CHECK_TIMEOUT: Duration = Duration::from_secs(2);
static ATTEMPT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn openat_file(parent: &File, name: &CStr, flags: i32) -> io::Result<File> {
    // SAFETY: `parent` is a live directory fd and `name` is a valid CStr.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags | libc::O_CLOEXEC | libc::O_NOFOLLOW) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        // SAFETY: successful `openat` returned a unique owned file descriptor.
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

fn bounded_read_file(file: File, max_bytes: usize) -> io::Result<String> {
    let mut bytes = String::new();
    file.take((max_bytes as u64).saturating_add(1)).read_to_string(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(io::Error::other("bounded proc/cgroup read limit exceeded"));
    }
    Ok(bytes)
}

fn bounded_cgroup_read(directory: &File, name: &CStr) -> io::Result<String> {
    let file = openat_file(directory, name, libc::O_RDONLY)?;
    bounded_read_file(file, MAX_CGROUP_READ_BYTES)
}

fn named_counter(text: &str, key: &str) -> io::Result<u64> {
    let mut counter = None;
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words.next() == Some(key) {
            if counter.is_some() {
                return Err(io::Error::other(format!("duplicate cgroup counter {key}")));
            }
            let value = words
                .next()
                .and_then(|n| n.parse::<u64>().ok())
                .ok_or_else(|| io::Error::other(format!("invalid cgroup counter {key}")))?;
            if words.next().is_some() {
                return Err(io::Error::other(format!("trailing tokens for cgroup counter {key}")));
            }
            counter = Some(value);
        }
    }
    counter.ok_or_else(|| io::Error::other(format!("missing cgroup counter {key}")))
}

fn parse_self_cgroup_v2_path() -> io::Result<String> {
    let file = File::open("/proc/self/cgroup")?;
    let content = bounded_read_file(file, MAX_CGROUP_READ_BYTES)?;
    extract_cgroup_v2_path(&content)
}

fn parse_pid_cgroup_v2_path(pid: u32) -> io::Result<String> {
    let path = format!("/proc/{pid}/cgroup");
    let file = File::open(path)?;
    let content = bounded_read_file(file, MAX_CGROUP_READ_BYTES)?;
    extract_cgroup_v2_path(&content)
}

fn extract_cgroup_v2_path(content: &str) -> io::Result<String> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("0::") {
            let trimmed = rest.trim();
            if trimmed.is_empty() {
                return Ok("/".to_string());
            }
            if !trimmed.starts_with('/') {
                return Ok(format!("/{trimmed}"));
            }
            return Ok(trimmed.to_string());
        }
    }
    Err(io::Error::other("process has no cgroup v2 entry (0::)"))
}

pub(crate) fn is_cgroup_path_contained(candidate: &str, container: &str) -> bool {
    if container == "/" {
        return candidate.starts_with('/');
    }
    if candidate == container {
        return true;
    }
    candidate.strip_prefix(container).is_some_and(|suffix| suffix.starts_with('/'))
}

fn unescape_mountinfo_field(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\'
            && i + 3 < bytes.len()
            && let Ok(oct_str) = std::str::from_utf8(&bytes[i + 1..i + 4])
            && oct_str.bytes().all(|b| (b'0'..=b'7').contains(&b))
            && let Ok(val) = u8::from_str_radix(oct_str, 8)
        {
            out.push(val as char);
            i += 4;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

struct Cgroup2MountInfo {
    mount_root: String,
    mount_point: PathBuf,
    root_rel_path: String,
}

fn resolve_cgroup2_mount_and_rel(root_fd: &File) -> io::Result<Cgroup2MountInfo> {
    let fd_link = PathBuf::from(format!("/proc/self/fd/{}", root_fd.as_raw_fd()));
    let resolved_root = fs::read_link(fd_link)?;
    let mountinfo_file = File::open("/proc/self/mountinfo")?;
    let mountinfo = bounded_read_file(mountinfo_file, MAX_MOUNTINFO_READ_BYTES)?;

    let mut best: Option<(usize, String, PathBuf)> = None;
    for line in mountinfo.lines() {
        let Some((left, right)) = line.split_once(" - ") else {
            continue;
        };
        let mut right_fields = right.split_whitespace();
        let Some(fstype) = right_fields.next() else {
            continue;
        };
        if fstype != "cgroup2" {
            continue;
        }
        let left_fields: Vec<&str> = left.split_whitespace().collect();
        if left_fields.len() < 5 {
            continue;
        }
        let mount_root = unescape_mountinfo_field(left_fields[3]);
        let mount_point = PathBuf::from(unescape_mountinfo_field(left_fields[4]));
        if let Ok(suffix) = resolved_root.strip_prefix(&mount_point) {
            let depth = mount_point.as_os_str().len();
            if best.as_ref().is_none_or(|(best_depth, _, _)| depth > *best_depth) {
                best = Some((depth, mount_root, mount_point.clone()));
                let _ = suffix;
            }
        }
    }

    let Some((_, mount_root, mount_point)) = best else {
        return Err(io::Error::other("cgroup root does not reside under a mounted cgroup2 filesystem"));
    };
    let suffix = resolved_root
        .strip_prefix(&mount_point)
        .map_err(|_| io::Error::other("cgroup root path escaped mount point"))?;
    let root_rel_path = join_cgroup_rel(&mount_root, suffix);
    Ok(Cgroup2MountInfo {
        mount_root,
        mount_point,
        root_rel_path,
    })
}

fn join_cgroup_rel(base: &str, suffix: &Path) -> String {
    let mut parts: Vec<String> = base.split('/').filter(|s| !s.is_empty()).map(str::to_string).collect();
    for component in suffix.components() {
        if let std::path::Component::Normal(seg) = component {
            parts.push(seg.to_string_lossy().into_owned());
        }
    }
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn lca_cgroup_rel(left: &str, right: &str) -> String {
    let left_parts: Vec<&str> = left.split('/').filter(|s| !s.is_empty()).collect();
    let right_parts: Vec<&str> = right.split('/').filter(|s| !s.is_empty()).collect();
    let mut common = Vec::new();
    for (l, r) in left_parts.iter().zip(right_parts.iter()) {
        if l == r {
            common.push(*l);
        } else {
            break;
        }
    }
    if common.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", common.join("/"))
    }
}

fn check_lca_migration_writable(mount: &Cgroup2MountInfo, self_cg_rel: &str) -> Result<(), SandboxUnavailableReason> {
    let lca = lca_cgroup_rel(self_cg_rel, &mount.root_rel_path);
    if !is_cgroup_path_contained(&lca, &mount.mount_root) {
        return Err(SandboxUnavailableReason::DelegationLcaNotWritable {
            self_cgroup: self_cg_rel.to_string(),
            target_cgroup: mount.root_rel_path.clone(),
        });
    }
    let rel_from_mount = if mount.mount_root == "/" {
        lca.trim_start_matches('/')
    } else {
        lca.strip_prefix(&mount.mount_root).unwrap_or("").trim_start_matches('/')
    };
    let lca_dir = if rel_from_mount.is_empty() {
        mount.mount_point.clone()
    } else {
        mount.mount_point.join(rel_from_mount)
    };
    let procs_path = lca_dir.join("cgroup.procs");
    let open_res = fs::OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&procs_path);
    if open_res.is_err() {
        return Err(SandboxUnavailableReason::DelegationLcaNotWritable {
            self_cgroup: self_cg_rel.to_string(),
            target_cgroup: mount.root_rel_path.clone(),
        });
    }
    Ok(())
}

fn parse_effective_u64_limit(raw: &str) -> io::Result<SandboxEffectiveLimit> {
    let trimmed = raw.trim();
    if trimmed == "max" {
        return Ok(SandboxEffectiveLimit::Unlimited);
    }
    let parsed = trimmed
        .parse::<u64>()
        .map_err(|_| io::Error::other(format!("unexpected cgroup limit value {trimmed:?}")))?;
    Ok(SandboxEffectiveLimit::Bytes(parsed))
}

fn parse_effective_u32_limit(raw: &str) -> io::Result<SandboxEffectiveLimit> {
    let trimmed = raw.trim();
    if trimmed == "max" {
        return Ok(SandboxEffectiveLimit::Unlimited);
    }
    let parsed = trimmed
        .parse::<u32>()
        .map_err(|_| io::Error::other(format!("unexpected cgroup count limit value {trimmed:?}")))?;
    Ok(SandboxEffectiveLimit::Count(parsed))
}

fn check_sandbox_binaries(bwrap_program: &Path, provide_shell: Option<&Path>) -> Result<(), SandboxUnavailableReason> {
    let bwrap_exists = if bwrap_program.components().count() > 1 || bwrap_program.is_absolute() {
        bwrap_program.is_file()
    } else {
        std::env::var_os("PATH")
            .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(bwrap_program).is_file()))
    };
    if !bwrap_exists {
        return Err(SandboxUnavailableReason::MissingSandboxBinaryOrShell {
            binary: bwrap_program.display().to_string(),
        });
    }
    if let Some(shell) = provide_shell
        && !shell.is_file()
    {
        return Err(SandboxUnavailableReason::MissingSandboxBinaryOrShell {
            binary: shell.display().to_string(),
        });
    }
    Ok(())
}

fn check_helper_socket(
    socket_kind: &'static str,
    host_socket: &Path,
    optional_if_missing: bool,
    attempt_cg_rel: &str,
    policy: SandboxHelperScopePolicy,
) -> Result<SandboxHelperScopeOutcome, SandboxUnavailableReason> {
    match fs::symlink_metadata(host_socket) {
        Ok(meta) if meta.file_type().is_socket() => {}
        Ok(_) => {
            return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                socket_kind: socket_kind.to_string(),
                detail: format!("endpoint {} is not a Unix socket", host_socket.display()),
            });
        }
        Err(err) if err.kind() == ErrorKind::NotFound && optional_if_missing => {
            return Ok(SandboxHelperScopeOutcome::NotAttached {
                socket_kind: socket_kind.to_string(),
            });
        }
        Err(err) => {
            return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                socket_kind: socket_kind.to_string(),
                detail: format!("failed to inspect socket {}: {err}", host_socket.display()),
            });
        }
    }

    match UnixStream::connect(host_socket) {
        Ok(stream) => {
            let _ = stream.set_read_timeout(Some(MAX_HELPER_CHECK_TIMEOUT));
            let _ = stream.set_write_timeout(Some(MAX_HELPER_CHECK_TIMEOUT));
            // SAFETY: `ucred` is POD and `getsockopt` writes exactly `len` bytes.
            let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
            let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
            let rc = unsafe {
                libc::getsockopt(
                    stream.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_PEERCRED,
                    (&mut cred as *mut libc::ucred).cast(),
                    &mut len,
                )
            };
            if rc != 0 || len as usize != std::mem::size_of::<libc::ucred>() || cred.pid <= 0 {
                return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                    socket_kind: socket_kind.to_string(),
                    detail: format!("SO_PEERCRED unavailable for {}", host_socket.display()),
                });
            }
            let euid = unsafe { libc::geteuid() };
            if cred.uid != euid {
                return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                    socket_kind: socket_kind.to_string(),
                    detail: format!(
                        "helper peer pid={} uid={} does not match current uid={}",
                        cred.pid, cred.uid, euid
                    ),
                });
            }
            let peer_pid = u32::try_from(cred.pid).unwrap_or(0);
            let peer_cg = parse_pid_cgroup_v2_path(peer_pid).map_err(|err| {
                SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                    socket_kind: socket_kind.to_string(),
                    detail: format!("failed to read /proc/{peer_pid}/cgroup: {err}"),
                }
            })?;
            if !is_cgroup_path_contained(&peer_cg, attempt_cg_rel) {
                return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                    socket_kind: socket_kind.to_string(),
                    detail: format!(
                        "helper peer pid={peer_pid} cgroup '{peer_cg}' is outside enforced attempt cgroup '{attempt_cg_rel}'"
                    ),
                });
            }
            Ok(SandboxHelperScopeOutcome::ContainedInAttemptCgroup {
                socket_kind: socket_kind.to_string(),
                peer_pid,
                peer_cgroup: peer_cg,
            })
        }
        Err(err) if err.raw_os_error() == Some(libc::EPROTOTYPE) => {
            // Non-stream Unix socket (e.g. SOCK_DGRAM WaitCollector). Verify
            // ownership strictly within /proc/self/fd without scanning host PIDs.
            let owned_in_process = is_dgram_socket_owned_in_process(socket_kind, host_socket)?;
            if !owned_in_process {
                return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                    socket_kind: socket_kind.to_string(),
                    detail: format!(
                        "datagram helper socket {} is owned outside the sandbox supervisor process",
                        host_socket.display()
                    ),
                });
            }
            match policy {
                SandboxHelperScopePolicy::RequireContained => {
                    Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
                        socket_kind: socket_kind.to_string(),
                        detail: format!(
                            "in-process datagram socket {} is outside enforced attempt cgroup '{attempt_cg_rel}' under RequireContained policy",
                            host_socket.display()
                        ),
                    })
                }
                SandboxHelperScopePolicy::AllowInProcessPassiveDatagram => {
                    Ok(SandboxHelperScopeOutcome::InProcessPassiveDatagram {
                        socket_kind: socket_kind.to_string(),
                    })
                }
            }
        }
        Err(err) => Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
            socket_kind: socket_kind.to_string(),
            detail: format!("failed to connect to helper socket {}: {err}", host_socket.display()),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HostSocketVfsId {
    encoded_dev: u32,
    ino: u64,
}

enum NetlinkDiagResult {
    FoundVfs { encoded_dev: u32, ino: u32 },
    FoundWithoutVfs,
    NotFound,
    Unavailable,
}

fn encode_linux_dev(dev: libc::dev_t) -> u32 {
    // Linux `UNIX_DIAG_VFS.udiag_vfs_dev` reports `sb->s_dev` (`MKDEV(major, minor)`
    // with `MINORBITS = 20`), whereas userspace `stat.st_dev` uses `new_encode_dev`.
    let major = libc::major(dev);
    let minor = libc::minor(dev);
    (major << 20) | (minor & ((1 << 20) - 1))
}

fn open_host_socket_vfs_id(
    socket_kind: &'static str,
    host_socket: &Path,
) -> Result<HostSocketVfsId, SandboxUnavailableReason> {
    let c_path = CString::new(host_socket.as_os_str().as_bytes()).map_err(|_| {
        SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
            socket_kind: socket_kind.to_string(),
            detail: format!("socket path {} contains interior NUL", host_socket.display()),
        }
    })?;
    let mut opened_fd: Option<OwnedFd> = None;
    for _ in 0..8 {
        // SAFETY: `c_path` is NUL-terminated; `O_PATH | O_NOFOLLOW | O_CLOEXEC`
        // opens the dentry itself without traversing a final symlink.
        let raw = unsafe { libc::open(c_path.as_ptr(), libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
        if raw >= 0 {
            // SAFETY: `raw` is a fresh owned fd from `open`.
            opened_fd = Some(unsafe { OwnedFd::from_raw_fd(raw) });
            break;
        }
        let err = io::Error::last_os_error();
        if err.kind() == ErrorKind::Interrupted {
            continue;
        }
        return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
            socket_kind: socket_kind.to_string(),
            detail: format!("failed to open datagram socket {}: {err}", host_socket.display()),
        });
    }
    let fd = opened_fd.ok_or_else(|| SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
        socket_kind: socket_kind.to_string(),
        detail: format!("interrupted opening datagram socket {}", host_socket.display()),
    })?;
    // SAFETY: `stat_buf` is POD zeroed before `fstat`.
    let mut stat_buf: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(fd.as_raw_fd(), &mut stat_buf) } != 0 {
        let err = io::Error::last_os_error();
        return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
            socket_kind: socket_kind.to_string(),
            detail: format!("failed to fstat datagram socket {}: {err}", host_socket.display()),
        });
    }
    if (stat_buf.st_mode & libc::S_IFMT) != libc::S_IFSOCK {
        return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
            socket_kind: socket_kind.to_string(),
            detail: format!("endpoint {} is not a non-symlink Unix socket", host_socket.display()),
        });
    }
    let euid = unsafe { libc::geteuid() };
    if stat_buf.st_uid != euid {
        return Err(SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope {
            socket_kind: socket_kind.to_string(),
            detail: format!(
                "datagram helper socket {} uid={} does not match current uid={euid}",
                host_socket.display(),
                stat_buf.st_uid
            ),
        });
    }
    Ok(HostSocketVfsId {
        encoded_dev: encode_linux_dev(stat_buf.st_dev),
        ino: stat_buf.st_ino,
    })
}

fn canonicalize_socket_path(path: &Path) -> Option<PathBuf> {
    let file_name = path.file_name()?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let canonical_parent = fs::canonicalize(parent).ok()?;
    Some(canonical_parent.join(file_name))
}

fn parse_proc_self_fd_socket_ino(link_target: &Path) -> Option<u64> {
    let text = link_target.to_str()?;
    let inner = text.strip_prefix("socket:[")?.strip_suffix(']')?;
    inner.parse::<u64>().ok()
}

fn read_verified_dgram_unix_path(fd: RawFd, expected_sock_ino: u64) -> Option<PathBuf> {
    // SAFETY: `stat_buf` is POD zeroed before `fstat`.
    let mut stat_buf: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(fd, &mut stat_buf) } != 0 {
        return None;
    }
    if (stat_buf.st_mode & libc::S_IFMT) != libc::S_IFSOCK || stat_buf.st_ino != expected_sock_ino {
        return None;
    }
    let int_size = libc::socklen_t::try_from(std::mem::size_of::<libc::c_int>()).ok()?;
    let mut domain: libc::c_int = 0;
    let mut opt_len = int_size;
    // SAFETY: `domain` is valid writable storage for `opt_len` bytes.
    if unsafe {
        libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_DOMAIN, (&mut domain as *mut libc::c_int).cast(), &mut opt_len)
    } != 0
        || domain != libc::AF_UNIX
    {
        return None;
    }
    let mut sock_type: libc::c_int = 0;
    let mut type_len = int_size;
    // SAFETY: `sock_type` is valid writable storage for `type_len` bytes.
    if unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_TYPE,
            (&mut sock_type as *mut libc::c_int).cast(),
            &mut type_len,
        )
    } != 0
        || sock_type != libc::SOCK_DGRAM
    {
        return None;
    }

    // SAFETY: `addr` is POD zeroed before `getsockname`.
    let mut addr: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    let mut addr_len = libc::socklen_t::try_from(std::mem::size_of::<libc::sockaddr_un>()).ok()?;
    // SAFETY: `addr` has capacity `addr_len`.
    if unsafe { libc::getsockname(fd, (&mut addr as *mut libc::sockaddr_un).cast(), &mut addr_len) } != 0 {
        return None;
    }
    if i32::from(addr.sun_family) != libc::AF_UNIX {
        return None;
    }
    let sun_path_offset = std::mem::offset_of!(libc::sockaddr_un, sun_path);
    let addr_len_usize = usize::try_from(addr_len).ok()?;
    if addr_len_usize <= sun_path_offset {
        return None;
    }
    let raw_len = (addr_len_usize - sun_path_offset).min(addr.sun_path.len());
    let raw_bytes: Vec<u8> = addr.sun_path[..raw_len].iter().map(|c| c.to_ne_bytes()[0]).collect();
    if raw_bytes.is_empty() || raw_bytes[0] == 0 {
        return None;
    }
    let trimmed = match raw_bytes.iter().position(|&b| b == 0) {
        Some(pos) => &raw_bytes[..pos],
        None => &raw_bytes[..],
    };
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(std::ffi::OsStr::from_bytes(trimmed)))
}

fn query_unix_diag_vfs(sock_ino: u32) -> NetlinkDiagResult {
    // SAFETY: creates a netlink SOCK_DIAG socket with SOCK_CLOEXEC.
    let raw_nl =
        unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW | libc::SOCK_CLOEXEC, libc::NETLINK_SOCK_DIAG) };
    if raw_nl < 0 {
        return NetlinkDiagResult::Unavailable;
    }
    // SAFETY: `raw_nl` is a freshly created owned fd.
    let nl_fd = unsafe { OwnedFd::from_raw_fd(raw_nl) };
    let timeout = libc::timeval { tv_sec: 1, tv_usec: 0 };
    let tv_len = match libc::socklen_t::try_from(std::mem::size_of::<libc::timeval>()) {
        Ok(len) => len,
        Err(_) => return NetlinkDiagResult::Unavailable,
    };
    // SAFETY: `timeout` is valid for `tv_len` bytes.
    let _ = unsafe {
        libc::setsockopt(
            nl_fd.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            (&timeout as *const libc::timeval).cast(),
            tv_len,
        )
    };

    let mut req = [0_u8; 40];
    req[0..4].copy_from_slice(&40_u32.to_ne_bytes());
    req[4..6].copy_from_slice(&SOCK_DIAG_BY_FAMILY.to_ne_bytes());
    let nl_flags = u16::try_from(libc::NLM_F_REQUEST).unwrap_or(1);
    req[6..8].copy_from_slice(&nl_flags.to_ne_bytes());
    req[8..12].copy_from_slice(&1_u32.to_ne_bytes());
    req[16] = u8::try_from(libc::AF_UNIX).unwrap_or(1);
    req[20..24].copy_from_slice(&u32::MAX.to_ne_bytes());
    req[24..28].copy_from_slice(&sock_ino.to_ne_bytes());
    req[28..32].copy_from_slice(&UDIAG_SHOW_VFS.to_ne_bytes());
    req[32..36].copy_from_slice(&u32::MAX.to_ne_bytes());
    req[36..40].copy_from_slice(&u32::MAX.to_ne_bytes());

    let mut sent = false;
    for _ in 0..8 {
        // SAFETY: `req` is a 40-byte stack buffer sent to the netlink socket.
        let rc = unsafe { libc::send(nl_fd.as_raw_fd(), req.as_ptr().cast(), req.len(), 0) };
        if rc == 40 {
            sent = true;
            break;
        }
        if rc < 0 && io::Error::last_os_error().kind() == ErrorKind::Interrupted {
            continue;
        }
        return NetlinkDiagResult::Unavailable;
    }
    if !sent {
        return NetlinkDiagResult::Unavailable;
    }

    let mut buf = [0_u8; 4096];
    for _ in 0..8 {
        // SAFETY: `buf` is a 4096-byte writable buffer.
        let n = unsafe { libc::recv(nl_fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len(), 0) };
        if n < 0 {
            if io::Error::last_os_error().kind() == ErrorKind::Interrupted {
                continue;
            }
            return NetlinkDiagResult::Unavailable;
        }
        let Ok(received) = usize::try_from(n) else {
            return NetlinkDiagResult::Unavailable;
        };
        return parse_unix_diag_response(&buf[..received], sock_ino);
    }
    NetlinkDiagResult::Unavailable
}

fn parse_unix_diag_response(bytes: &[u8], expected_sock_ino: u32) -> NetlinkDiagResult {
    let mut offset = 0usize;
    while offset + 16 <= bytes.len() {
        let Ok(len_bytes) = bytes[offset..offset + 4].try_into() else {
            break;
        };
        let msg_len = u32::from_ne_bytes(len_bytes) as usize;
        if msg_len < 16 || offset + msg_len > bytes.len() {
            break;
        }
        let Ok(type_bytes) = bytes[offset + 4..offset + 6].try_into() else {
            break;
        };
        let msg_type = u16::from_ne_bytes(type_bytes);
        if msg_type == NLMSG_ERROR_TYPE {
            if msg_len >= 20 {
                let Ok(err_bytes) = bytes[offset + 16..offset + 20].try_into() else {
                    break;
                };
                let nl_err = i32::from_ne_bytes(err_bytes);
                if nl_err != 0 {
                    let errno = nl_err.saturating_neg();
                    if errno == libc::ENOENT {
                        return NetlinkDiagResult::NotFound;
                    }
                    return NetlinkDiagResult::Unavailable;
                }
            }
        } else if msg_type == NLMSG_DONE_TYPE {
            break;
        } else if msg_type == SOCK_DIAG_BY_FAMILY && msg_len >= 32 {
            let family = i32::from(bytes[offset + 16]);
            let sock_type = i32::from(bytes[offset + 17]);
            let Ok(ino_bytes) = bytes[offset + 20..offset + 24].try_into() else {
                break;
            };
            let diag_ino = u32::from_ne_bytes(ino_bytes);
            if family == libc::AF_UNIX && sock_type == libc::SOCK_DGRAM && diag_ino == expected_sock_ino {
                let msg_end = offset + msg_len;
                let mut rta_off = offset + 32;
                while rta_off + 4 <= msg_end {
                    let Ok(rta_len_bytes) = bytes[rta_off..rta_off + 2].try_into() else {
                        break;
                    };
                    let Ok(rta_type_bytes) = bytes[rta_off + 2..rta_off + 4].try_into() else {
                        break;
                    };
                    let rta_len = usize::from(u16::from_ne_bytes(rta_len_bytes));
                    let rta_type = u16::from_ne_bytes(rta_type_bytes);
                    if rta_len < 4 || rta_off + rta_len > msg_end {
                        break;
                    }
                    if rta_type == UNIX_DIAG_VFS && rta_len >= 12 {
                        let Ok(vfs_ino_bytes) = bytes[rta_off + 4..rta_off + 8].try_into() else {
                            break;
                        };
                        let Ok(vfs_dev_bytes) = bytes[rta_off + 8..rta_off + 12].try_into() else {
                            break;
                        };
                        return NetlinkDiagResult::FoundVfs {
                            encoded_dev: u32::from_ne_bytes(vfs_dev_bytes),
                            ino: u32::from_ne_bytes(vfs_ino_bytes),
                        };
                    }
                    rta_off += (rta_len + 3) & !3;
                }
                return NetlinkDiagResult::FoundWithoutVfs;
            }
        }
        offset += (msg_len + 3) & !3;
    }
    NetlinkDiagResult::NotFound
}

fn parse_proc_net_unix_line(line: &str) -> Option<(u64, &str, Option<&str>)> {
    let mut rest = line.trim_start_matches([' ', '\t']);
    let mut fields = [""; 7];
    for (idx, slot) in fields.iter_mut().enumerate() {
        match rest.find([' ', '\t']) {
            Some(sep_idx) => {
                *slot = &rest[..sep_idx];
                rest = rest[sep_idx..].trim_start_matches([' ', '\t']);
            }
            None if idx == 6 && !rest.is_empty() => {
                *slot = rest;
                rest = "";
            }
            None => return None,
        }
    }
    if fields[0] == "Num" || fields[4].is_empty() || fields[6].is_empty() {
        return None;
    }
    let ino = fields[6].parse::<u64>().ok()?;
    let path = (!rest.is_empty()).then_some(rest);
    Some((ino, fields[4], path))
}

fn proc_net_unix_has_unique_dgram_owner(content: &str, sock_ino: u64, canonical_host_socket: &Path) -> bool {
    let mut found_target = false;
    for line in content.lines().take(MAX_PROC_NET_UNIX_LINES) {
        let Some((ino, sock_type, opt_path)) = parse_proc_net_unix_line(line) else {
            continue;
        };
        if sock_type != "0002" {
            continue;
        }
        if ino == sock_ino {
            found_target = true;
            continue;
        }
        if let Some(other_path_str) = opt_path
            && !other_path_str.starts_with('@')
        {
            let other_path = Path::new(other_path_str);
            if other_path == canonical_host_socket
                || canonicalize_socket_path(other_path).as_deref() == Some(canonical_host_socket)
            {
                return false;
            }
        }
    }
    found_target
}

fn check_proc_net_unix_dgram_fallback(bound_path: &Path, sock_ino: u64, canonical_host_socket: Option<&Path>) -> bool {
    if !bound_path.is_absolute() {
        return false;
    }
    let Some(canonical_host) = canonical_host_socket else {
        return false;
    };
    let Some(canonical_bound) = canonicalize_socket_path(bound_path) else {
        return false;
    };
    if canonical_bound != canonical_host {
        return false;
    }
    let Ok(file) = File::open("/proc/net/unix") else {
        return false;
    };
    let Ok(content) = bounded_read_file(file, MAX_PROC_NET_UNIX_BYTES) else {
        return false;
    };
    proc_net_unix_has_unique_dgram_owner(&content, sock_ino, canonical_host)
}

fn is_dgram_socket_owned_in_process(
    socket_kind: &'static str,
    host_socket: &Path,
) -> Result<bool, SandboxUnavailableReason> {
    let target = open_host_socket_vfs_id(socket_kind, host_socket)?;
    let canonical_host = canonicalize_socket_path(host_socket);
    let Ok(entries) = fs::read_dir("/proc/self/fd") else {
        return Ok(false);
    };
    for entry in entries.flatten().take(MAX_PROC_SELF_FD_SCAN) {
        let Some(fd_str) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(raw_fd) = fd_str.parse::<RawFd>() else {
            continue;
        };
        if raw_fd < 0 {
            continue;
        }
        let Ok(link_target) = fs::read_link(entry.path()) else {
            continue;
        };
        let Some(sock_ino) = parse_proc_self_fd_socket_ino(&link_target) else {
            continue;
        };
        let Some(bound_path) = read_verified_dgram_unix_path(raw_fd, sock_ino) else {
            continue;
        };
        if let Ok(sock_ino_u32) = u32::try_from(sock_ino) {
            match query_unix_diag_vfs(sock_ino_u32) {
                NetlinkDiagResult::FoundVfs { encoded_dev, ino } => {
                    let ino_matches = encoded_dev == target.encoded_dev && u64::from(ino) == (target.ino & 0xFFFF_FFFF);
                    if !ino_matches {
                        continue;
                    }
                    if target.ino > u64::from(u32::MAX)
                        && canonical_host
                            .as_deref()
                            .is_none_or(|host| canonicalize_socket_path(&bound_path).as_deref() != Some(host))
                    {
                        continue;
                    }
                    return Ok(true);
                }
                NetlinkDiagResult::FoundWithoutVfs | NetlinkDiagResult::NotFound => {
                    continue;
                }
                NetlinkDiagResult::Unavailable => {}
            }
        }
        if check_proc_net_unix_dgram_fallback(&bound_path, sock_ino, canonical_host.as_deref()) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) struct AttemptCgroup {
    root: File,
    directory: File,
    name: CString,
    rel_path: String,
    memory_active: bool,
    pids_active: bool,
}

pub(crate) struct PendingCgroupSpawn {
    _procs_fd: File,
    _ack_writer: UnixStream,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CgroupPostExitEvents {
    pub(crate) oom_count: u64,
    pub(crate) oom_kill_count: u64,
    pub(crate) memory_peak_bytes: Option<u64>,
    pub(crate) pids_max_events: u64,
}

impl AttemptCgroup {
    pub(crate) fn rel_path(&self) -> &str {
        &self.rel_path
    }

    pub(crate) fn is_diagnostics_only(&self) -> bool {
        !self.memory_active && !self.pids_active
    }

    pub(crate) fn read_cpu_usage_usec(&self) -> io::Result<u64> {
        let text = bounded_cgroup_read(&self.directory, c"cpu.stat")?;
        named_counter(&text, "usage_usec")
    }

    pub(crate) fn configure_pre_exec(&self, command: &mut Command) -> io::Result<(PendingCgroupSpawn, UnixStream)> {
        let procs_fd = openat_file(&self.directory, c"cgroup.procs", libc::O_WRONLY)?;
        let (ack_reader, ack_writer) = UnixStream::pair()?;
        let procs_raw = procs_fd.as_raw_fd();
        let ack_raw = ack_writer.as_raw_fd();
        let diagnostics_only = self.is_diagnostics_only();
        // SAFETY: `pre_exec` runs between `fork` and `execve` while both
        // `procs_raw` and `ack_raw` remain owned by `PendingCgroupSpawn` in the
        // parent and open in the forked child. Only async-signal-safe `libc::write`
        // is executed on success; on failure we write `b"0"` and return `Err`
        // (or `Ok(())` in diagnostics-only mode so the build proceeds without failing).
        unsafe {
            command.pre_exec(move || {
                let zero = b"0";
                let written = libc::write(procs_raw, zero.as_ptr().cast(), zero.len());
                if written == 1 {
                    let ok = b"1";
                    let acked = libc::write(ack_raw, ok.as_ptr().cast(), ok.len());
                    if acked == 1 {
                        return Ok(());
                    }
                }
                let err = io::Error::last_os_error();
                let fail = b"0";
                let _ = libc::write(ack_raw, fail.as_ptr().cast(), fail.len());
                if diagnostics_only { Ok(()) } else { Err(err) }
            });
        }
        Ok((
            PendingCgroupSpawn {
                _procs_fd: procs_fd,
                _ack_writer: ack_writer,
            },
            ack_reader,
        ))
    }

    pub(crate) fn wait_migration_ack(
        &self,
        mut ack_reader: UnixStream,
        timeout: Duration,
    ) -> Result<(), SandboxUnavailableReason> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(SandboxUnavailableReason::ChildMigrationFailed {
                    detail: "timed out waiting for pre_exec cgroup migration acknowledgement".to_string(),
                });
            }
            let timeout_ms = i32::try_from(remaining.as_millis()).unwrap_or(i32::MAX).max(1);
            let mut pfd = libc::pollfd {
                fd: ack_reader.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: `pfd` points to one live stack `pollfd` and `ack_reader` is owned.
            let rc = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
            if rc < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == ErrorKind::Interrupted {
                    continue;
                }
                return Err(SandboxUnavailableReason::ChildMigrationFailed {
                    detail: format!("poll on cgroup migration ack failed: {err}"),
                });
            }
            if rc == 0 {
                continue;
            }
            let mut buf = [0_u8; 1];
            match ack_reader.read(&mut buf) {
                Ok(1) if buf[0] == b'1' => return Ok(()),
                Ok(1) => {
                    return Err(SandboxUnavailableReason::ChildMigrationFailed {
                        detail: "child pre_exec reported cgroup migration failure".to_string(),
                    });
                }
                Ok(_) => {
                    return Err(SandboxUnavailableReason::ChildMigrationFailed {
                        detail: "child closed cgroup migration ack pipe without confirmation".to_string(),
                    });
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => {
                    return Err(SandboxUnavailableReason::ChildMigrationFailed {
                        detail: format!("reading cgroup migration ack failed: {err}"),
                    });
                }
            }
        }
    }

    pub(crate) fn observe_live_membership(&self, facts: &mut SandboxResourceEnforcementFacts) {
        if let Ok(procs) = bounded_cgroup_read(&self.directory, c"cgroup.procs") {
            let pids: Vec<u32> = procs.split_whitespace().filter_map(|s| s.parse::<u32>().ok()).collect();
            if !pids.is_empty() {
                facts.observed_member_pids = pids;
            }
        }
        if let Ok(events) = bounded_cgroup_read(&self.directory, c"cgroup.events")
            && named_counter(&events, "populated").unwrap_or(0) > 0
        {
            facts.populated_observed = true;
        }
        facts.attempt_cgroup_rel_path = Some(self.rel_path.clone());
    }

    pub(crate) fn drain_and_collect(&self, timeout: Duration) -> io::Result<CgroupPostExitEvents> {
        let deadline = Instant::now() + timeout;
        let mut kill_attempted = false;
        loop {
            let populated = bounded_cgroup_read(&self.directory, c"cgroup.events")
                .and_then(|txt| named_counter(&txt, "populated"))?;
            if populated == 0 {
                break;
            }
            if Instant::now() >= deadline {
                if !kill_attempted {
                    kill_attempted = true;
                    if let Ok(mut kill_file) = openat_file(&self.directory, c"cgroup.kill", libc::O_WRONLY) {
                        let _ = kill_file.write_all(b"1");
                    }
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                return Err(io::Error::other("attempt cgroup remained populated after teardown"));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        if !bounded_cgroup_read(&self.directory, c"cgroup.procs")?.trim().is_empty() {
            return Err(io::Error::other("attempt cgroup.procs not empty after populated == 0"));
        }
        let mut collected = CgroupPostExitEvents::default();
        if self.memory_active {
            let events = bounded_cgroup_read(&self.directory, c"memory.events")?;
            collected.oom_count = named_counter(&events, "oom").unwrap_or(0);
            collected.oom_kill_count = named_counter(&events, "oom_kill").unwrap_or(0);
            collected.memory_peak_bytes =
                bounded_cgroup_read(&self.directory, c"memory.peak").ok().and_then(|s| s.trim().parse::<u64>().ok());
        }
        if self.pids_active
            && let Ok(events) = bounded_cgroup_read(&self.directory, c"pids.events")
        {
            collected.pids_max_events = named_counter(&events, "max").unwrap_or(0);
        }
        Ok(collected)
    }
}

impl Drop for AttemptCgroup {
    fn drop(&mut self) {
        // SAFETY: removes only the fresh per-attempt child directory inside `root`.
        unsafe {
            libc::unlinkat(self.root.as_raw_fd(), self.name.as_ptr(), libc::AT_REMOVEDIR);
        }
    }
}

pub(crate) struct PreparedAttemptEnforcement {
    pub(crate) cgroup: Option<Arc<AttemptCgroup>>,
    pub(crate) facts: SandboxResourceEnforcementFacts,
}

pub(crate) struct AttemptEnforcementInput<'a> {
    pub(crate) policy: &'a SandboxResourcePolicy,
    pub(crate) bwrap_program: &'a Path,
    pub(crate) provide_shell: Option<&'a Path>,
    pub(crate) has_bind_scratches: bool,
    pub(crate) compile_socket: Option<&'a Path>,
    pub(crate) rust_wait_socket: Option<&'a Path>,
    pub(crate) pidfd_injected_unavailable: bool,
}

pub(crate) fn prepare_attempt_enforcement(
    input: AttemptEnforcementInput<'_>,
) -> Result<PreparedAttemptEnforcement, Box<(SandboxUnavailableReason, SandboxResourceEnforcementFacts)>> {
    let mut facts = SandboxResourceEnforcementFacts {
        swap_policy: input.policy.swap_policy,
        ..SandboxResourceEnforcementFacts::default()
    };

    // Always evaluate binary/shell & pidfd capabilities if any enforcement is requested.
    if input.policy.requests_any_enforcement() {
        if input.pidfd_injected_unavailable || super::watch_cancel::require_pidfd_support().is_err() {
            let reason = SandboxUnavailableReason::KernelFeatureUnsupported {
                feature: "pidfd_open".to_string(),
            };
            facts.unavailable_reasons.push(reason.clone());
            if input.policy.requires_cgroup() || input.policy.scratch_mode == SandboxEnforcementMode::Required {
                return Err(Box::new((reason, facts)));
            }
        }
        if let Err(reason) = check_sandbox_binaries(input.bwrap_program, input.provide_shell) {
            facts.unavailable_reasons.push(reason.clone());
            if input.policy.requires_cgroup() || input.policy.scratch_mode == SandboxEnforcementMode::Required {
                return Err(Box::new((reason, facts)));
            }
        }
    }

    // Scratch enforcement evaluation: host `--bind` scratch directories have no kernel size cap.
    if input.policy.scratch_mode != SandboxEnforcementMode::Disabled && input.has_bind_scratches {
        let reason = SandboxUnavailableReason::ScratchBindDirUnenforced;
        facts.unavailable_reasons.push(reason.clone());
        if input.policy.scratch_mode == SandboxEnforcementMode::Required {
            return Err(Box::new((reason, facts)));
        }
    }

    if !input.policy.requests_cgroup() {
        return Ok(PreparedAttemptEnforcement { cgroup: None, facts });
    }

    match setup_attempt_cgroup(&input, &mut facts) {
        Ok(cgroup) => Ok(PreparedAttemptEnforcement {
            cgroup: Some(Arc::new(cgroup)),
            facts,
        }),
        Err(reason) => {
            facts.enforced_memory = false;
            facts.enforced_pids = false;
            facts.effective_memory_max = SandboxEffectiveLimit::NotConfigured;
            facts.effective_swap_max = SandboxEffectiveLimit::NotConfigured;
            facts.effective_oom_group = None;
            facts.effective_pids_max = SandboxEffectiveLimit::NotConfigured;
            if !facts.unavailable_reasons.contains(&reason) {
                facts.unavailable_reasons.push(reason.clone());
            }
            if input.policy.requires_cgroup() {
                Err(Box::new((reason, facts)))
            } else {
                Ok(PreparedAttemptEnforcement { cgroup: None, facts })
            }
        }
    }
}

pub(crate) fn prepare_attempt_diagnostics(
    cgroup_root: Option<&Path>,
) -> Result<AttemptCgroup, SandboxLiveCpuUnavailableReason> {
    let attempt = create_verified_attempt_cgroup_dir(cgroup_root, false, false).map_err(unavailable_for_live_cpu)?;
    if let Err(err) = attempt.read_cpu_usage_usec() {
        return Err(SandboxLiveCpuUnavailableReason::CpuStatUnreadable {
            detail: format!("initial cpu.stat read failed on {}: {err}", attempt.rel_path()),
        });
    }
    Ok(attempt)
}

pub(crate) fn unavailable_for_live_cpu(reason: SandboxUnavailableReason) -> SandboxLiveCpuUnavailableReason {
    match reason {
        SandboxUnavailableReason::CgroupRootUnconfigured => SandboxLiveCpuUnavailableReason::CgroupRootUnconfigured,
        SandboxUnavailableReason::CgroupRootInvalidOrUnowned { detail } => {
            SandboxLiveCpuUnavailableReason::CgroupRootInvalidOrUnowned { detail }
        }
        SandboxUnavailableReason::DelegationLcaNotWritable {
            self_cgroup,
            target_cgroup,
        } => SandboxLiveCpuUnavailableReason::DelegationLcaNotWritable {
            detail: format!("self_cgroup={self_cgroup} target_cgroup={target_cgroup}"),
        },
        SandboxUnavailableReason::ChildMigrationFailed { detail } => {
            SandboxLiveCpuUnavailableReason::ChildMigrationFailed { detail }
        }
        SandboxUnavailableReason::ControllerDisabled { controller } => SandboxLiveCpuUnavailableReason::SetupIoError {
            detail: format!("controller disabled: {controller}"),
        },
        SandboxUnavailableReason::ControllerSubtreeBlocked { controller, detail } => {
            SandboxLiveCpuUnavailableReason::SetupIoError {
                detail: format!("controller {controller} subtree blocked: {detail}"),
            }
        }
        SandboxUnavailableReason::KernelFeatureUnsupported { feature } => {
            SandboxLiveCpuUnavailableReason::SetupIoError {
                detail: format!("kernel feature unsupported: {feature}"),
            }
        }
        SandboxUnavailableReason::MissingSandboxBinaryOrShell { binary } => {
            SandboxLiveCpuUnavailableReason::SetupIoError {
                detail: format!("missing sandbox binary: {binary}"),
            }
        }
        SandboxUnavailableReason::ScratchBindDirUnenforced => SandboxLiveCpuUnavailableReason::SetupIoError {
            detail: "scratch bind dir unenforced".to_string(),
        },
        SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { socket_kind, detail } => {
            SandboxLiveCpuUnavailableReason::SetupIoError {
                detail: format!("helper {socket_kind} outside enforced scope: {detail}"),
            }
        }
        SandboxUnavailableReason::SetupIoError { detail } => SandboxLiveCpuUnavailableReason::SetupIoError { detail },
    }
}

fn create_verified_attempt_cgroup_dir(
    cgroup_root: Option<&Path>,
    need_memory: bool,
    need_pids: bool,
) -> Result<AttemptCgroup, SandboxUnavailableReason> {
    let Some(root_path) = cgroup_root else {
        return Err(SandboxUnavailableReason::CgroupRootUnconfigured);
    };
    let root_file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root_path)
        .map_err(|err| SandboxUnavailableReason::CgroupRootInvalidOrUnowned {
            detail: format!("opening cgroup root {}: {err}", root_path.display()),
        })?;
    let meta = root_file.metadata().map_err(|err| SandboxUnavailableReason::CgroupRootInvalidOrUnowned {
        detail: format!("stat cgroup root {}: {err}", root_path.display()),
    })?;
    let euid = unsafe { libc::geteuid() };
    if meta.uid() != euid || (meta.mode() & 0o022) != 0 {
        return Err(SandboxUnavailableReason::CgroupRootInvalidOrUnowned {
            detail: format!(
                "cgroup root {} uid={} (expected {euid}) mode={:#o}",
                root_path.display(),
                meta.uid(),
                meta.mode() & 0o777
            ),
        });
    }
    let mut statfs_buf: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatfs(root_file.as_raw_fd(), &mut statfs_buf) } != 0
        || (statfs_buf.f_type as u64) != CGROUP2_SUPER_MAGIC
    {
        return Err(SandboxUnavailableReason::CgroupRootInvalidOrUnowned {
            detail: format!("{} is not a cgroup v2 filesystem", root_path.display()),
        });
    }

    let mount = resolve_cgroup2_mount_and_rel(&root_file).map_err(|err| {
        SandboxUnavailableReason::CgroupRootInvalidOrUnowned {
            detail: err.to_string(),
        }
    })?;
    let self_cg_rel = parse_self_cgroup_v2_path().map_err(|err| SandboxUnavailableReason::SetupIoError {
        detail: err.to_string(),
    })?;
    check_lca_migration_writable(&mount, &self_cg_rel)?;

    if need_memory || need_pids {
        check_and_enable_controllers(&root_file, need_memory, need_pids)?;
    }

    let seq = ATTEMPT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name_str = format!("mantle-sbx-{}-{seq}", std::process::id());
    let name = CString::new(name_str.clone()).map_err(|_| SandboxUnavailableReason::SetupIoError {
        detail: "invalid cgroup name".to_string(),
    })?;
    if unsafe { libc::mkdirat(root_file.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
        let err = io::Error::last_os_error();
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: format!("mkdirat {name_str}: {err}"),
        });
    }
    let directory = match openat_file(&root_file, &name, libc::O_RDONLY | libc::O_DIRECTORY) {
        Ok(dir) => dir,
        Err(err) => {
            unsafe {
                libc::unlinkat(root_file.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR);
            }
            return Err(SandboxUnavailableReason::SetupIoError {
                detail: format!("openat {name_str}: {err}"),
            });
        }
    };
    let attempt_rel_path = if mount.root_rel_path == "/" {
        format!("/{name_str}")
    } else {
        format!("{}/{name_str}", mount.root_rel_path.trim_end_matches('/'))
    };
    let attempt = AttemptCgroup {
        root: root_file,
        directory,
        name,
        rel_path: attempt_rel_path.clone(),
        memory_active: need_memory,
        pids_active: need_pids,
    };

    if unsafe { libc::fchmod(attempt.directory.as_raw_fd(), 0o700) } != 0 {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: format!("fchmod on {attempt_rel_path} failed"),
        });
    }
    let procs = bounded_cgroup_read(&attempt.directory, c"cgroup.procs").map_err(|err| {
        SandboxUnavailableReason::SetupIoError {
            detail: format!("read {attempt_rel_path}/cgroup.procs: {err}"),
        }
    })?;
    if !procs.trim().is_empty() {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "fresh attempt cgroup is not empty".to_string(),
        });
    }

    Ok(attempt)
}

fn setup_attempt_cgroup(
    input: &AttemptEnforcementInput<'_>,
    facts: &mut SandboxResourceEnforcementFacts,
) -> Result<AttemptCgroup, SandboxUnavailableReason> {
    let need_memory = input.policy.memory_mode != SandboxEnforcementMode::Disabled;
    let need_pids = input.policy.pids_mode != SandboxEnforcementMode::Disabled;
    let attempt = create_verified_attempt_cgroup_dir(input.policy.cgroup_root.as_deref(), need_memory, need_pids)?;
    let attempt_rel_path = attempt.rel_path.clone();

    if need_memory {
        configure_memory_controller(&attempt, input.policy, facts)?;
    }
    if need_pids {
        configure_pids_controller(&attempt, input.policy, facts)?;
    }

    // Verify helper/daemon sockets belong to the enforced attempt cgroup.
    if let Some(compile_sock) = input.compile_socket {
        let outcome = check_helper_socket(
            "compile-cache",
            compile_sock,
            true,
            &attempt_rel_path,
            input.policy.helper_scope_policy,
        )?;
        facts.helper_checks.push(outcome);
    }
    if let Some(wait_sock) = input.rust_wait_socket {
        let outcome =
            check_helper_socket("rust-waits", wait_sock, false, &attempt_rel_path, input.policy.helper_scope_policy)?;
        facts.helper_checks.push(outcome);
    }

    facts.attempt_cgroup_rel_path = Some(attempt_rel_path);
    Ok(attempt)
}

fn check_and_enable_controllers(
    root_file: &File,
    need_memory: bool,
    need_pids: bool,
) -> Result<(), SandboxUnavailableReason> {
    let available_raw = bounded_cgroup_read(root_file, c"cgroup.controllers").map_err(|err| {
        SandboxUnavailableReason::SetupIoError {
            detail: format!("reading cgroup.controllers: {err}"),
        }
    })?;
    let available: BTreeSet<&str> = available_raw.split_whitespace().collect();
    let subtree_raw = bounded_cgroup_read(root_file, c"cgroup.subtree_control").map_err(|err| {
        SandboxUnavailableReason::SetupIoError {
            detail: format!("reading cgroup.subtree_control: {err}"),
        }
    })?;
    let subtree: BTreeSet<&str> = subtree_raw.split_whitespace().collect();

    for (needed, ctrl) in [(need_memory, "memory"), (need_pids, "pids")] {
        if !needed {
            continue;
        }
        if !available.contains(ctrl) {
            return Err(SandboxUnavailableReason::ControllerDisabled {
                controller: ctrl.to_string(),
            });
        }
        if !subtree.contains(ctrl) {
            let mut file = openat_file(root_file, c"cgroup.subtree_control", libc::O_WRONLY).map_err(|err| {
                SandboxUnavailableReason::ControllerSubtreeBlocked {
                    controller: ctrl.to_string(),
                    detail: format!("opening cgroup.subtree_control: {err}"),
                }
            })?;
            let enable_token = format!("+{ctrl}");
            file.write_all(enable_token.as_bytes()).map_err(|err| {
                SandboxUnavailableReason::ControllerSubtreeBlocked {
                    controller: ctrl.to_string(),
                    detail: format!("enabling {enable_token} on cgroup.subtree_control: {err}"),
                }
            })?;
        }
    }
    Ok(())
}

fn configure_memory_controller(
    attempt: &AttemptCgroup,
    policy: &SandboxResourcePolicy,
    facts: &mut SandboxResourceEnforcementFacts,
) -> Result<(), SandboxUnavailableReason> {
    let Some(limit_bytes) = policy.memory_limit_bytes else {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "memory enforcement enabled without memory_limit_bytes".to_string(),
        });
    };
    let events = bounded_cgroup_read(&attempt.directory, c"memory.events").map_err(|_| {
        SandboxUnavailableReason::ControllerDisabled {
            controller: "memory".to_string(),
        }
    })?;
    if named_counter(&events, "oom_kill").unwrap_or(1) != 0 || named_counter(&events, "oom").unwrap_or(1) != 0 {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "fresh attempt cgroup has non-zero initial memory.events".to_string(),
        });
    }

    let mut oom_group = openat_file(&attempt.directory, c"memory.oom.group", libc::O_WRONLY).map_err(|_| {
        SandboxUnavailableReason::KernelFeatureUnsupported {
            feature: "memory.oom.group".to_string(),
        }
    })?;
    oom_group.write_all(b"1").map_err(|_| SandboxUnavailableReason::KernelFeatureUnsupported {
        feature: "memory.oom.group".to_string(),
    })?;

    let mut max_file = openat_file(&attempt.directory, c"memory.max", libc::O_WRONLY).map_err(|_| {
        SandboxUnavailableReason::ControllerDisabled {
            controller: "memory".to_string(),
        }
    })?;
    max_file
        .write_all(limit_bytes.to_string().as_bytes())
        .map_err(|err| SandboxUnavailableReason::SetupIoError {
            detail: format!("writing memory.max: {err}"),
        })?;

    match policy.swap_policy {
        SandboxSwapPolicy::HostDefault => {}
        SandboxSwapPolicy::Zero => {
            let mut swap_file = openat_file(&attempt.directory, c"memory.swap.max", libc::O_WRONLY).map_err(|_| {
                SandboxUnavailableReason::KernelFeatureUnsupported {
                    feature: "memory.swap.max".to_string(),
                }
            })?;
            swap_file.write_all(b"0").map_err(|_| SandboxUnavailableReason::KernelFeatureUnsupported {
                feature: "memory.swap.max".to_string(),
            })?;
        }
        SandboxSwapPolicy::BoundedBytes { swap_limit_bytes } => {
            let mut swap_file = openat_file(&attempt.directory, c"memory.swap.max", libc::O_WRONLY).map_err(|_| {
                SandboxUnavailableReason::KernelFeatureUnsupported {
                    feature: "memory.swap.max".to_string(),
                }
            })?;
            swap_file.write_all(swap_limit_bytes.to_string().as_bytes()).map_err(|_| {
                SandboxUnavailableReason::KernelFeatureUnsupported {
                    feature: "memory.swap.max".to_string(),
                }
            })?;
        }
    }

    let read_max = bounded_cgroup_read(&attempt.directory, c"memory.max")
        .and_then(|s| parse_effective_u64_limit(&s))
        .map_err(|err| SandboxUnavailableReason::SetupIoError {
        detail: format!("reading back memory.max: {err}"),
    })?;
    let read_oom_group = bounded_cgroup_read(&attempt.directory, c"memory.oom.group")
        .map(|s| s.trim() == "1")
        .map_err(|err| SandboxUnavailableReason::SetupIoError {
            detail: format!("reading back memory.oom.group: {err}"),
        })?;
    let read_swap = match bounded_cgroup_read(&attempt.directory, c"memory.swap.max") {
        Ok(s) => parse_effective_u64_limit(&s).unwrap_or(SandboxEffectiveLimit::NotConfigured),
        Err(_) => SandboxEffectiveLimit::NotConfigured,
    };

    facts.effective_memory_max = read_max;
    facts.effective_swap_max = read_swap;
    facts.effective_oom_group = Some(read_oom_group);
    facts.enforced_memory = matches!(facts.effective_memory_max, SandboxEffectiveLimit::Bytes(v) if v <= limit_bytes);
    if !facts.enforced_memory {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "effective memory.max did not reflect configured limit".to_string(),
        });
    }
    Ok(())
}

fn configure_pids_controller(
    attempt: &AttemptCgroup,
    policy: &SandboxResourcePolicy,
    facts: &mut SandboxResourceEnforcementFacts,
) -> Result<(), SandboxUnavailableReason> {
    let Some(limit_count) = policy.pids_limit_count else {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "pids enforcement enabled without pids_limit_count".to_string(),
        });
    };
    let current = bounded_cgroup_read(&attempt.directory, c"pids.current").map_err(|_| {
        SandboxUnavailableReason::ControllerDisabled {
            controller: "pids".to_string(),
        }
    })?;
    if current.trim() != "0" {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "fresh attempt cgroup has non-zero pids.current".to_string(),
        });
    }
    let mut pids_max = openat_file(&attempt.directory, c"pids.max", libc::O_WRONLY).map_err(|_| {
        SandboxUnavailableReason::ControllerDisabled {
            controller: "pids".to_string(),
        }
    })?;
    pids_max
        .write_all(limit_count.to_string().as_bytes())
        .map_err(|err| SandboxUnavailableReason::SetupIoError {
            detail: format!("writing pids.max: {err}"),
        })?;
    let read_pids = bounded_cgroup_read(&attempt.directory, c"pids.max")
        .and_then(|s| parse_effective_u32_limit(&s))
        .map_err(|err| SandboxUnavailableReason::SetupIoError {
            detail: format!("reading back pids.max: {err}"),
        })?;
    facts.effective_pids_max = read_pids;
    facts.enforced_pids = matches!(facts.effective_pids_max, SandboxEffectiveLimit::Count(v) if v <= limit_count);
    if !facts.enforced_pids {
        return Err(SandboxUnavailableReason::SetupIoError {
            detail: "effective pids.max did not reflect configured limit".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgroup_path_containment_handles_exact_child_and_prefix_collisions() {
        assert!(is_cgroup_path_contained("/a/b", "/a/b"));
        assert!(is_cgroup_path_contained("/a/b/c", "/a/b"));
        assert!(!is_cgroup_path_contained("/a/b2", "/a/b"));
        assert!(!is_cgroup_path_contained("/a", "/a/b"));
        assert!(is_cgroup_path_contained("/anything", "/"));
    }

    #[test]
    fn extract_cgroup_v2_path_handles_standard_and_root_lines() {
        assert_eq!(extract_cgroup_v2_path("0::/user.slice/foo\n").unwrap(), "/user.slice/foo");
        assert_eq!(extract_cgroup_v2_path("0::/\n").unwrap(), "/");
        assert_eq!(extract_cgroup_v2_path("0::\n").unwrap(), "/");
        assert!(extract_cgroup_v2_path("1:cpu:/foo\n").is_err());
    }

    #[test]
    fn unconfigured_cgroup_root_reports_explicit_unavailable_and_never_fakes_enforcement() {
        let policy = SandboxResourcePolicy {
            cgroup_root: None,
            memory_mode: SandboxEnforcementMode::Required,
            memory_limit_bytes: Some(64 * 1024 * 1024),
            ..SandboxResourcePolicy::disabled()
        };
        let err = prepare_attempt_enforcement(AttemptEnforcementInput {
            policy: &policy,
            bwrap_program: Path::new("/bin/sh"),
            provide_shell: None,
            has_bind_scratches: true,
            compile_socket: None,
            rust_wait_socket: None,
            pidfd_injected_unavailable: false,
        })
        .err()
        .expect("missing cgroup root must fail closed under Required mode");
        assert_eq!(err.0, SandboxUnavailableReason::CgroupRootUnconfigured);
        assert!(!err.1.enforced_memory);
    }

    #[test]
    fn scratch_bind_enforcement_required_reports_scratch_bind_dir_unenforced() {
        let policy = SandboxResourcePolicy {
            scratch_mode: SandboxEnforcementMode::Required,
            scratch_limit_bytes: Some(1024 * 1024),
            ..SandboxResourcePolicy::disabled()
        };
        let err = prepare_attempt_enforcement(AttemptEnforcementInput {
            policy: &policy,
            bwrap_program: Path::new("/bin/sh"),
            provide_shell: None,
            has_bind_scratches: true,
            compile_socket: None,
            rust_wait_socket: None,
            pidfd_injected_unavailable: false,
        })
        .err()
        .expect("host bind scratch dirs cannot claim hard enforcement");
        assert_eq!(err.0, SandboxUnavailableReason::ScratchBindDirUnenforced);
        assert!(!err.1.enforced_scratch);
    }

    #[test]
    fn dgram_helper_socket_ownership_distinguishes_in_process_external_unlinked_and_seqpacket() {
        use std::io::BufRead;
        use std::io::BufReader;
        use std::os::unix::net::UnixDatagram;
        use std::process::Stdio;

        if let Some(sock_path) = std::env::var_os("MANTLE_TEST_EXTERNAL_DGRAM_SOCKET") {
            let _sock = UnixDatagram::bind(&sock_path).expect("external child must bind dgram socket");
            println!("READY");
            let _ = std::io::stdout().flush();
            let mut stdin_buf = [0_u8; 16];
            while let Ok(n) = std::io::stdin().read(&mut stdin_buf) {
                if n == 0 {
                    break;
                }
            }
            return;
        }

        let dir = tempfile::tempdir().unwrap();

        // 1. In-process UnixDatagram + AllowInProcessPassiveDatagram -> accepted.
        let in_proc_path = dir.path().join("in-proc.sock");
        let _in_proc_sock = UnixDatagram::bind(&in_proc_path).unwrap();
        let allow_outcome = check_helper_socket(
            "rust-waits",
            &in_proc_path,
            false,
            "/attempt",
            SandboxHelperScopePolicy::AllowInProcessPassiveDatagram,
        )
        .expect("in-process UnixDatagram must be allowed under AllowInProcessPassiveDatagram");
        assert_eq!(allow_outcome, SandboxHelperScopeOutcome::InProcessPassiveDatagram {
            socket_kind: "rust-waits".to_string(),
        });

        // 2. In-process UnixDatagram + RequireContained -> refused.
        let req_err = check_helper_socket(
            "rust-waits",
            &in_proc_path,
            false,
            "/attempt",
            SandboxHelperScopePolicy::RequireContained,
        )
        .unwrap_err();
        assert!(matches!(req_err, SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { .. }));

        fn await_external_ready(stdout: std::process::ChildStdout) -> BufReader<std::process::ChildStdout> {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            for _ in 0..32 {
                line.clear();
                let n = reader.read_line(&mut line).unwrap();
                assert!(n > 0, "external dgram helper exited before publishing READY");
                if line.trim() == "READY" {
                    return reader;
                }
            }
            panic!("external dgram helper did not publish READY within 32 stdout lines");
        }

        // 3. External-process UnixDatagram + AllowInProcessPassiveDatagram -> refused.
        let ext_path = dir.path().join("external.sock");
        let exe = std::env::current_exe().unwrap();
        let mut ext_child = std::process::Command::new(&exe)
            .env("MANTLE_TEST_EXTERNAL_DGRAM_SOCKET", &ext_path)
            .args([
                "--exact",
                "bwrap::cgroup::tests::dgram_helper_socket_ownership_distinguishes_in_process_external_unlinked_and_seqpacket",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let _ext_stdout = await_external_ready(ext_child.stdout.take().unwrap());

        let ext_err = check_helper_socket(
            "rust-waits",
            &ext_path,
            false,
            "/attempt",
            SandboxHelperScopePolicy::AllowInProcessPassiveDatagram,
        )
        .unwrap_err();
        assert!(matches!(ext_err, SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { .. }));
        drop(ext_child.stdin.take());
        assert!(ext_child.wait().unwrap().success());

        // 4. Unlinked-and-rebound attack: self holds unlinked S1 at rebind.sock; external child binds S2 at
        //    rebind.sock -> must refuse.
        let rebind_path = dir.path().join("rebind.sock");
        let _unlinked_self_sock = UnixDatagram::bind(&rebind_path).unwrap();
        fs::remove_file(&rebind_path).unwrap();
        let mut rebind_child = std::process::Command::new(&exe)
            .env("MANTLE_TEST_EXTERNAL_DGRAM_SOCKET", &rebind_path)
            .args([
                "--exact",
                "bwrap::cgroup::tests::dgram_helper_socket_ownership_distinguishes_in_process_external_unlinked_and_seqpacket",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let _rebind_stdout = await_external_ready(rebind_child.stdout.take().unwrap());

        let rebind_err = check_helper_socket(
            "rust-waits",
            &rebind_path,
            false,
            "/attempt",
            SandboxHelperScopePolicy::AllowInProcessPassiveDatagram,
        )
        .unwrap_err();
        assert!(matches!(rebind_err, SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { .. }));
        drop(rebind_child.stdin.take());
        assert!(rebind_child.wait().unwrap().success());

        // 5. In-process SOCK_SEQPACKET (also fails UnixStream::connect with EPROTOTYPE) -> refused.
        let seq_path = dir.path().join("seqpacket.sock");
        let raw_seq = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0) };
        assert!(raw_seq >= 0);
        let seq_fd = unsafe { OwnedFd::from_raw_fd(raw_seq) };
        let mut seq_addr: libc::sockaddr_un = unsafe { std::mem::zeroed() };
        seq_addr.sun_family = u16::try_from(libc::AF_UNIX).unwrap();
        let seq_bytes = seq_path.as_os_str().as_bytes();
        assert!(seq_bytes.len() < seq_addr.sun_path.len());
        for (idx, byte) in seq_bytes.iter().enumerate() {
            seq_addr.sun_path[idx] = libc::c_char::from_ne_bytes([*byte]);
        }
        let bind_len = u32::try_from(std::mem::offset_of!(libc::sockaddr_un, sun_path) + seq_bytes.len() + 1).unwrap();
        assert_eq!(
            unsafe { libc::bind(seq_fd.as_raw_fd(), (&seq_addr as *const libc::sockaddr_un).cast(), bind_len) },
            0
        );
        assert_eq!(unsafe { libc::listen(seq_fd.as_raw_fd(), 4) }, 0);
        let seq_err = check_helper_socket(
            "rust-waits",
            &seq_path,
            false,
            "/attempt",
            SandboxHelperScopePolicy::AllowInProcessPassiveDatagram,
        )
        .unwrap_err();
        assert!(matches!(seq_err, SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { .. }));

        // 6. Symlink to in-process UnixDatagram -> refused.
        let symlink_path = dir.path().join("symlink-to-in-proc.sock");
        std::os::unix::fs::symlink(&in_proc_path, &symlink_path).unwrap();
        let sym_err = check_helper_socket(
            "rust-waits",
            &symlink_path,
            false,
            "/attempt",
            SandboxHelperScopePolicy::AllowInProcessPassiveDatagram,
        )
        .unwrap_err();
        assert!(matches!(sym_err, SandboxUnavailableReason::HelperDaemonOutsideEnforcedScope { .. }));
    }

    #[test]
    fn proc_net_unix_fallback_handles_spaces_and_detects_rebound_conflicts() {
        let dir = tempfile::tempdir().unwrap();
        let space_dir = dir.path().join("dir with spaces");
        fs::create_dir_all(&space_dir).unwrap();
        let canonical_sock = canonicalize_socket_path(&space_dir.join("wait socket.sock")).unwrap();

        let unique_table = format!(
            "Num       RefCount Protocol Flags    Type St Inode Path\n\
             0000000000000000: 00000002 00000000 00010000 0001 01 11111 {}\n\
             0000000000000001: 00000002 00000000 00000000 0002 01 22222 {}\n",
            canonical_sock.display(),
            canonical_sock.display(),
        );
        assert!(proc_net_unix_has_unique_dgram_owner(&unique_table, 22222, &canonical_sock));
        assert!(!proc_net_unix_has_unique_dgram_owner(&unique_table, 99999, &canonical_sock));

        let rebound_conflict_table = format!(
            "Num       RefCount Protocol Flags    Type St Inode Path\n\
             0000000000000001: 00000002 00000000 00000000 0002 01 22222 {}\n\
             0000000000000002: 00000002 00000000 00000000 0002 01 33333 {}\n",
            canonical_sock.display(),
            canonical_sock.display(),
        );
        assert!(!proc_net_unix_has_unique_dgram_owner(&rebound_conflict_table, 22222, &canonical_sock));
    }

    #[test]
    fn live_cpu_diagnostics_unreadable_or_missing_cpu_stat_reports_unavailable_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        // Missing cgroup_root -> CgroupRootUnconfigured
        assert_eq!(
            prepare_attempt_diagnostics(None).err(),
            Some(SandboxLiveCpuUnavailableReason::CgroupRootUnconfigured)
        );
        // Non-cgroup2 directory -> CgroupRootInvalidOrUnowned
        assert!(matches!(
            prepare_attempt_diagnostics(Some(dir.path())).err(),
            Some(SandboxLiveCpuUnavailableReason::CgroupRootInvalidOrUnowned { .. })
        ));
        // Direct AttemptCgroup::read_cpu_usage_usec on missing/malformed cpu.stat -> Err without panic
        let root_file = File::open(dir.path()).unwrap();
        let attempt_dir_path = dir.path().join("fake-attempt");
        fs::create_dir(&attempt_dir_path).unwrap();
        let attempt_dir = File::open(&attempt_dir_path).unwrap();
        let attempt = AttemptCgroup {
            root: root_file,
            directory: attempt_dir,
            name: CString::new("fake-attempt").unwrap(),
            rel_path: "/fake-attempt".to_string(),
            memory_active: false,
            pids_active: false,
        };
        assert!(attempt.read_cpu_usage_usec().is_err());
        fs::write(attempt_dir_path.join("cpu.stat"), "nr_periods 0\n").unwrap();
        assert!(attempt.read_cpu_usage_usec().is_err());
        fs::write(attempt_dir_path.join("cpu.stat"), "usage_usec 42195\nuser_usec 12000\n").unwrap();
        assert_eq!(attempt.read_cpu_usage_usec().unwrap(), 42_195);
    }
}
