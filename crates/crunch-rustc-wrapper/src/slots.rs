// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::ambient_clock)]

//! Same-daemon compile slots. Scheduling receipts are not build evidence.
//!
//! Each lease occupies one dedicated, bounded daemon connection worker. Never
//! dispatch these connections on workers needed for cache completion/release.
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use std::time::Instant;

use crunch_rust_cache_core::slots::MAX_BUILD_ID_BYTES;
use crunch_rust_cache_core::slots::MAX_WAIT_TIMEOUT_MS;
use crunch_rust_cache_core::slots::SlotDecision;
use crunch_rust_cache_core::slots::SlotPolicy;
use crunch_rust_cache_core::slots::SlotReceipt;
use crunch_rust_cache_core::slots::SlotRejectReason;
use crunch_rust_cache_core::slots::SlotRequest;
use crunch_rust_cache_core::slots::SlotState;
use serde::Deserialize;
use serde::Serialize;

use crate::Error;
use crate::read_frame;
use crate::write_frame;

pub const SLOT_ACQUIRE_SCHEMA: &str = "mantle-compile-slot-acquire-v1";
pub const SLOT_DEGRADATION_SCHEMA: &str = "mantle-compile-slot-degraded-v1";
pub const SLOT_FRAME_BYTES: u64 = 4_096;
const CHECK_INTERVAL: Duration = Duration::from_millis(20);
const START_CHILD: u8 = 1;
const RELEASE: u8 = 2;
const CHILD_READY: u8 = 3;

type ReceiptSink = Arc<dyn Fn(&SlotReceipt) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotAcquireRequest {
    pub schema: String,
    pub build_id: String,
}

struct AuthorityState {
    slots: SlotState,
    next_request_id: u64,
}

/// Shared by the existing daemon's per-connection dispatch, not another service.
pub struct SlotAuthority {
    state: Mutex<AuthorityState>,
    changed: Condvar,
    epoch: Instant,
    receipt_sink: ReceiptSink,
    max_connections: u32,
}

// r[impl mantle.compile_slots.slot_authority]
impl SlotAuthority {
    pub fn new(policy: SlotPolicy) -> Result<Self, Error> {
        Self::with_receipt_sink(
            policy,
            Arc::new(|receipt| match serde_json::to_string(receipt) {
                Ok(json) => eprintln!("{json}"),
                Err(error) => eprintln!("compile-slot-receipt-encoding-failed:{error}"),
            }),
        )
    }

    /// The sink receives every transition, including dead-client reclamation.
    /// It must not panic. It is invoked without holding the authority mutex.
    pub fn with_receipt_sink(policy: SlotPolicy, receipt_sink: ReceiptSink) -> Result<Self, Error> {
        let slots =
            SlotState::new(policy).map_err(|reason| Error::Policy(format!("compile-slot-policy:{reason:?}")))?;
        let max_connections = slots.policy().total_slots.saturating_add(slots.policy().total_queue);
        Ok(Self {
            state: Mutex::new(AuthorityState {
                slots,
                next_request_id: 1,
            }),
            changed: Condvar::new(),
            epoch: Instant::now(),
            receipt_sink,
            max_connections,
        })
    }

    pub fn connection_limit(&self) -> u32 {
        self.max_connections
    }

    fn lock(&self) -> Result<MutexGuard<'_, AuthorityState>, Error> {
        self.state.lock().map_err(|_| Error::Process("compile-slot-lock-poisoned".to_string()))
    }

    fn now_ms(&self) -> u64 {
        let elapsed_ms = self.epoch.elapsed().as_millis();
        // A lease older than the u64 millisecond range saturates at the range end.
        u64::try_from(elapsed_ms).map_or(u64::MAX, |value| value)
    }

    fn emit(&self, receipt: &SlotReceipt) {
        (self.receipt_sink)(receipt);
    }

    fn reserve_available(&self) -> Result<(), Error> {
        let mut pending = self.lock()?.slots.reserve(self.now_ms());
        while pending.is_some() {
            if let Some(receipt) = pending.take() {
                self.changed.notify_all();
                self.emit(&receipt);
            }
            pending = self.lock()?.slots.reserve(self.now_ms());
        }
        Ok(())
    }

    /// Caller authenticates the peer using the existing daemon policy before
    /// dispatch. `request` is the already-read first framed JSON message.
    pub fn handle_connection(&self, stream: &mut UnixStream, request: SlotAcquireRequest) -> Result<(), Error> {
        if request.schema != SLOT_ACQUIRE_SCHEMA {
            return Err(Error::Protocol("compile-slot-request-schema".to_string()));
        }
        let admitted_ms = self.now_ms();
        let (request_id, timeout_ms, receipt) = {
            let mut state = self.lock()?;
            let request_id = state.next_request_id;
            state.next_request_id = request_id
                .checked_add(1)
                .ok_or_else(|| Error::Bound("compile-slot-request-id-exhausted".to_string()))?;
            let timeout_ms = state.slots.policy().wait_timeout_ms;
            let receipt = state.slots.admit(
                SlotRequest {
                    request_id,
                    build_id: request.build_id,
                },
                admitted_ms,
            );
            (request_id, timeout_ms, receipt)
        };
        self.emit(&receipt);
        if receipt.decision == SlotDecision::Rejected {
            return write_frame(stream, &receipt, SLOT_FRAME_BYTES);
        }
        let mut reservation = ServerReservation {
            authority: self,
            request_id,
            child: None,
            child_group: None,
            settled: false,
        };
        self.reserve_available()?;
        let grant = self.wait_for_grant(stream, request_id, admitted_ms, timeout_ms)?;
        if grant.decision != SlotDecision::Granted {
            reservation.settled = true;
            return write_frame(stream, &grant, SLOT_FRAME_BYTES);
        }
        let result = self.serve_lease(stream, &mut reservation, &grant, timeout_ms);
        if result.is_err() {
            // Unblock a forked child still waiting for ACK before waiting on its pidfd.
            let shutdown = stream.shutdown(std::net::Shutdown::Both);
            debug_assert!(shutdown.is_ok(), "compile-slot-shutdown-failed");
        }
        result
    }

    fn wait_for_grant(
        &self,
        stream: &UnixStream,
        request_id: u64,
        admitted_ms: u64,
        timeout_ms: u64,
    ) -> Result<SlotReceipt, Error> {
        let mut state = self.lock()?;
        loop {
            if let Some(receipt) = state.slots.grant(request_id) {
                return Ok(receipt);
            }
            let elapsed_ms = self.now_ms().saturating_sub(admitted_ms);
            let reason = if disconnected(stream) {
                Some(SlotRejectReason::Cancelled)
            } else if elapsed_ms >= timeout_ms {
                Some(SlotRejectReason::WaitTimeout)
            } else {
                None
            };
            if let Some(reason) = reason {
                let receipt = state
                    .slots
                    .cancel(request_id, self.now_ms(), reason)
                    .ok_or_else(|| Error::Protocol("compile-slot-cancel-missing".to_string()))?;
                drop(state);
                self.emit(&receipt);
                self.changed.notify_all();
                self.reserve_available()?;
                return Ok(receipt);
            }
            let remaining = Duration::from_millis(timeout_ms.saturating_sub(elapsed_ms)).min(CHECK_INTERVAL);
            state = self
                .changed
                .wait_timeout(state, remaining)
                .map_err(|_| Error::Process("compile-slot-wait-lock-poisoned".to_string()))?
                .0;
        }
    }

    fn serve_lease(
        &self,
        stream: &mut UnixStream,
        reservation: &mut ServerReservation<'_>,
        grant: &SlotReceipt,
        timeout_ms: u64,
    ) -> Result<(), Error> {
        stream.set_read_timeout(Some(Duration::from_millis(timeout_ms))).map_err(io_error)?;
        stream.set_write_timeout(Some(Duration::from_millis(timeout_ms))).map_err(io_error)?;
        enable_sender_credentials(stream)?;
        write_frame(stream, grant, SLOT_FRAME_BYTES)?;
        let (operation, sender_pid) = lifecycle_operation(stream)?;
        match operation {
            RELEASE => {}
            START_CHILD => {
                let mut bytes = [0_u8; 4];
                stream.read_exact(&mut bytes).map_err(io_error)?;
                let claimed_pid = u32::from_be_bytes(bytes);
                if claimed_pid == 0 {
                    return Err(Error::Protocol("compile-slot-child-pid-invalid".into()));
                }
                // SCM_CREDENTIALS is translated by the kernel into the daemon's
                // PID namespace. A sandbox's getpid() is not a host process ID.
                let pid = sender_pid;
                reservation.child = Some(open_pidfd(pid)?);
                // SAFETY: getpgid queries the live child blocked before exec.
                if unsafe { libc::getpgid(pid as i32) } != pid as i32 {
                    return Err(Error::Protocol("compile-slot-child-process-group".to_string()));
                }
                reservation.child_group = Some(pid as i32);
                // No compiler exec before the daemon owns its pidfd and PGID.
                stream.write_all(&[CHILD_READY]).map_err(io_error)?;
                if !await_release_or_disconnect(stream)? {
                    terminate_process_group(pid as i32);
                }
            }
            _ => return Err(Error::Protocol("compile-slot-lifecycle-opcode".to_string())),
        }
        let receipt = reservation.settle()?;
        write_frame(stream, &receipt, SLOT_FRAME_BYTES)
    }

    /// A bounded snapshot for shell contention fixtures and operator status.
    pub fn occupancy(&self) -> Result<(u32, u32), Error> {
        let state = self.lock()?;
        Ok((state.slots.active_count(), state.slots.queued_count()))
    }
}

struct ServerReservation<'a> {
    authority: &'a SlotAuthority,
    request_id: u64,
    child: Option<OwnedFd>,
    child_group: Option<i32>,
    settled: bool,
}

impl ServerReservation<'_> {
    fn settle(&mut self) -> Result<SlotReceipt, Error> {
        if let Some(child) = &self.child {
            wait_child(child);
        }
        if let Some(group) = self.child_group {
            wait_process_group(group);
        }
        let receipt = {
            let mut state = self.authority.lock()?;
            state
                .slots
                .release(self.request_id, self.authority.now_ms())
                .or_else(|| state.slots.cancel(self.request_id, self.authority.now_ms(), SlotRejectReason::Cancelled))
                .ok_or_else(|| Error::Protocol("compile-slot-release-missing".to_string()))?
        };
        self.settled = true;
        self.authority.emit(&receipt);
        self.authority.changed.notify_all();
        self.authority.reserve_available()?;
        Ok(receipt)
    }
}

impl Drop for ServerReservation<'_> {
    fn drop(&mut self) {
        if !self.settled {
            // A lost driver must not leave an orphan compiler tree occupying
            // capacity forever. Only the fresh, handshaken cooperative group
            // belongs to this lease; processes escaping it are out of scope.
            if let Some(group) = self.child_group {
                terminate_process_group(group);
            }
            if let Err(error) = self.settle() {
                eprintln!("compile-slot-reclaim-failed:{error}");
            }
        }
    }
}

fn enable_sender_credentials(stream: &UnixStream) -> Result<(), Error> {
    let enabled: libc::c_int = 1;
    // SAFETY: valid borrowed socket and correctly sized integer option value.
    let result = unsafe {
        libc::setsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PASSCRED,
            std::ptr::addr_of!(enabled).cast(),
            std::mem::size_of_val(&enabled) as libc::socklen_t,
        )
    };
    if result != 0 {
        return Err(io_error(io::Error::last_os_error()));
    }
    Ok(())
}

fn lifecycle_operation(stream: &UnixStream) -> Result<(u8, u32), Error> {
    let mut operation = 0_u8;
    let mut vector = libc::iovec {
        iov_base: std::ptr::addr_of_mut!(operation).cast(),
        iov_len: 1,
    };
    // usize alignment is sufficient for cmsghdr; the buffer exceeds CMSG_SPACE(ucred).
    let mut control = [0_usize; 8];
    // SAFETY: msghdr is plain data; all pointer fields are initialized below.
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = std::ptr::addr_of_mut!(vector);
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr().cast();
    message.msg_controllen = std::mem::size_of_val(&control);
    receive_lifecycle_byte(stream, &mut message)?;
    debug_assert_eq!(message.msg_iovlen, 1);
    if message.msg_flags & libc::MSG_CTRUNC != 0 {
        return Err(Error::Protocol("compile-slot-truncated-credentials".into()));
    }
    let sender_pid = sender_pid_from_credentials(&message)?;
    debug_assert!(sender_pid > 0);
    Ok((operation, sender_pid))
}

/// Reads the single lifecycle byte, retrying only on an interrupted syscall.
fn receive_lifecycle_byte(stream: &UnixStream, message: &mut libc::msghdr) -> Result<(), Error> {
    let mut is_received = false;
    while !is_received {
        // SAFETY: message owns correctly sized writable buffers throughout recvmsg.
        let count = unsafe { libc::recvmsg(stream.as_raw_fd(), message, 0) };
        if count < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
            continue;
        }
        if count != 1 {
            return Err(io_error(if count == 0 {
                io::Error::from(io::ErrorKind::UnexpectedEof)
            } else {
                io::Error::last_os_error()
            }));
        }
        is_received = true;
    }
    Ok(())
}

/// Reads the sender's host PID from the ancillary credential header.
fn sender_pid_from_credentials(message: &libc::msghdr) -> Result<u32, Error> {
    // SAFETY: CMSG_LEN is a pure size computation over a compile-time constant.
    let credential_len = unsafe { libc::CMSG_LEN(std::mem::size_of::<libc::ucred>() as libc::c_uint) };
    let credential_min_bytes = usize::try_from(credential_len).map_or(usize::MAX, |min_bytes| min_bytes);
    debug_assert!(credential_min_bytes > 0);
    // SAFETY: libc traverses the returned ancillary buffer using its recorded bounds.
    let mut header = unsafe { libc::CMSG_FIRSTHDR(message) };
    while !header.is_null() {
        // SAFETY: header is a valid control header inside message's stack buffer.
        let credential = unsafe {
            if (*header).cmsg_level == libc::SOL_SOCKET
                && (*header).cmsg_type == libc::SCM_CREDENTIALS
                && (*header).cmsg_len >= credential_min_bytes
            {
                Some(std::ptr::read_unaligned(libc::CMSG_DATA(header).cast::<libc::ucred>()))
            } else {
                None
            }
        };
        if let Some(credential) = credential {
            if credential.pid <= 0 {
                return Err(Error::Protocol("compile-slot-invisible-sender".into()));
            }
            debug_assert!(credential.pid > 0);
            return u32::try_from(credential.pid).map_err(|_| Error::Protocol("compile-slot-invisible-sender".into()));
        }
        // SAFETY: current header and message are valid ancillary buffer views.
        header = unsafe { libc::CMSG_NXTHDR(message, header) };
    }
    Err(Error::Protocol("compile-slot-missing-sender-credentials".into()))
}

fn io_error(source: io::Error) -> Error {
    Error::Io {
        context: "compile-slot-transport".to_string(),
        source,
    }
}

fn disconnected(stream: &UnixStream) -> bool {
    let mut byte = 0_u8;
    // SAFETY: one writable byte and a valid borrowed socket descriptor.
    let count = unsafe {
        libc::recv(stream.as_raw_fd(), std::ptr::addr_of_mut!(byte).cast(), 1, libc::MSG_PEEK | libc::MSG_DONTWAIT)
    };
    count == 0
        || (count < 0
            && !matches!(io::Error::last_os_error().kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted))
}

fn open_pidfd(pid: u32) -> Result<OwnedFd, Error> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(Error::Protocol("compile-slot-child-pid-invalid".to_string()));
    }
    // SAFETY: pidfd_open takes an integer pid and zero flags, returning a new fd.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0_u32) };
    if fd < 0 {
        return Err(io_error(io::Error::last_os_error()));
    }
    let descriptor = i32::try_from(fd).map_err(|_| io_error(io::Error::from_raw_os_error(libc::EOVERFLOW)))?;
    // SAFETY: successful pidfd_open returned a newly owned descriptor.
    Ok(unsafe { OwnedFd::from_raw_fd(descriptor) })
}

/// Fail closed on transient poll errors: never return while the child is live.
fn wait_child(child: &OwnedFd) {
    let mut is_live = true;
    while is_live {
        let mut descriptor = libc::pollfd {
            fd: child.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: descriptor is a valid one-element poll array; pidfd stays owned.
        let result = unsafe { libc::poll(&mut descriptor, 1, -1) };
        if result > 0 && descriptor.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
            is_live = false;
        } else if result < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            std::thread::sleep(CHECK_INTERVAL);
        }
    }
}

fn await_release_or_disconnect(stream: &mut UnixStream) -> Result<bool, Error> {
    let mut operation = [0_u8; 1];
    let mut is_released = false;
    let mut is_disconnected = false;
    while !is_released && !is_disconnected {
        match stream.read(&mut operation) {
            Ok(0) => is_disconnected = true,
            Ok(_) if operation[0] == RELEASE => is_released = true,
            Ok(_) => return Err(Error::Protocol("compile-slot-release-opcode".to_string())),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(io_error(error)),
        }
    }
    debug_assert!(is_released != is_disconnected);
    debug_assert!(!is_released || operation[0] == RELEASE);
    Ok(is_released)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotDegradedReason {
    InvalidRequest,
    AuthorityUnavailable,
    AuthorityTimeout,
    AuthorityProtocol,
    AuthorityRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotDegradation {
    pub schema: String,
    pub build_id: String,
    pub disposition: String,
    pub reason: SlotDegradedReason,
    pub authority_receipt: Option<SlotReceipt>,
}

#[derive(Debug)]
pub enum SlotClientOutcome {
    Granted(SlotLease),
    Degraded(SlotDegradation),
}

/// One compile only. Dropping the guard disconnects; the daemon terminates and
/// waits for the owned cooperative compiler group before reclaiming the slot.
#[derive(Debug)]
pub struct SlotLease {
    stream: UnixStream,
    pub grant_receipt: SlotReceipt,
    spawn_attempted: bool,
}

// r[impl mantle.compile_slots.degrade_without_authority]
pub fn acquire_slot(socket: &Path, build_id: &str, wait_timeout: Duration) -> SlotClientOutcome {
    let is_build_id_invalid = build_id.is_empty() || build_id.len() > MAX_BUILD_ID_BYTES;
    let is_wait_budget_invalid = wait_timeout.is_zero() || wait_timeout > Duration::from_millis(MAX_WAIT_TIMEOUT_MS);
    if is_build_id_invalid || is_wait_budget_invalid {
        return degraded_outcome(build_id, SlotDegradedReason::InvalidRequest, None);
    }
    debug_assert!(!build_id.is_empty());
    debug_assert!(build_id.len() <= MAX_BUILD_ID_BYTES);
    debug_assert!(!wait_timeout.is_zero());
    let started = Instant::now();
    let mut stream = match connect_bounded(socket, wait_timeout) {
        Ok(stream) => stream,
        Err(_) => return degraded_outcome(build_id, SlotDegradedReason::AuthorityUnavailable, None),
    };
    let mut deadline_io = DeadlineIo::new(&mut stream, started + wait_timeout);
    let request = SlotAcquireRequest {
        schema: SLOT_ACQUIRE_SCHEMA.to_string(),
        build_id: build_id.to_string(),
    };
    if write_frame(&mut deadline_io, &request, SLOT_FRAME_BYTES).is_err() {
        return degraded_outcome(build_id, SlotDegradedReason::AuthorityUnavailable, None);
    }
    let response = read_frame::<SlotReceipt>(&mut deadline_io, SLOT_FRAME_BYTES);
    match response {
        Ok(receipt) if receipt.is_grant_for(build_id) => SlotClientOutcome::Granted(SlotLease {
            stream,
            grant_receipt: receipt,
            spawn_attempted: false,
        }),
        Ok(receipt) => degraded_outcome(build_id, SlotDegradedReason::AuthorityRejected, Some(receipt)),
        Err(Error::Io { source, .. })
            if matches!(source.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) =>
        {
            degraded_outcome(build_id, SlotDegradedReason::AuthorityTimeout, None)
        }
        Err(_) => degraded_outcome(build_id, SlotDegradedReason::AuthorityProtocol, None),
    }
}

/// Builds the degraded outcome published when the authority cannot grant.
fn degraded_outcome(
    build_id: &str,
    reason: SlotDegradedReason,
    authority_receipt: Option<SlotReceipt>,
) -> SlotClientOutcome {
    SlotClientOutcome::Degraded(SlotDegradation {
        schema: SLOT_DEGRADATION_SCHEMA.to_string(),
        build_id: build_id.to_string(),
        disposition: "degraded".to_string(),
        reason,
        authority_receipt,
    })
}

pub(crate) struct DeadlineIo<'a> {
    stream: &'a mut UnixStream,
    deadline: Instant,
}

impl<'a> DeadlineIo<'a> {
    pub(crate) fn new(stream: &'a mut UnixStream, deadline: Instant) -> Self {
        Self { stream, deadline }
    }

    fn remaining(&self) -> io::Result<Duration> {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::from(io::ErrorKind::TimedOut));
        }
        Ok(remaining)
    }
}

impl Read for DeadlineIo<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.stream.set_read_timeout(Some(self.remaining()?))?;
        self.stream.read(bytes)
    }
}

impl Write for DeadlineIo<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.stream.set_write_timeout(Some(self.remaining()?))?;
        self.stream.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl SlotLease {
    /// Preserves all caller-selected stdio (including bounded-output tempfiles).
    /// The Command is single-use: do not reuse it after this method installs its
    /// pre-exec hook. No allocation, locking, or Rust I/O occurs after fork.
    /// On spawn error, drop the lease and compile using a fresh unleased Command
    /// with an explicit degraded receipt; the failed child never reached exec.
    pub fn spawn(&mut self, command: &mut Command) -> io::Result<Child> {
        if self.spawn_attempted {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "compile-slot-lease-already-used"));
        }
        let timeout_ms = self.grant_receipt.wait_timeout_ms.clamp(1, MAX_WAIT_TIMEOUT_MS);
        self.stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;
        self.stream.set_write_timeout(Some(Duration::from_millis(timeout_ms)))?;
        self.spawn_attempted = true;
        let descriptor = self.stream.as_raw_fd();
        // SAFETY: hook calls only async-signal-safe setpgid/getpid/send/recv. The lease
        // owns this fd throughout spawn and Command must not be reused.
        unsafe {
            command.pre_exec(move || child_handshake(descriptor));
        }
        command.spawn()
    }

    /// Call after Child::wait and output cleanup. Early release still cannot
    /// free a live child: the daemon independently waits on its pidfd.
    pub fn release(mut self) -> Result<SlotReceipt, Error> {
        self.stream.write_all(&[RELEASE]).map_err(io_error)?;
        read_frame(&mut self.stream, SLOT_FRAME_BYTES)
    }
}

/// Compiler children inherit a fresh group. A zombie cannot execute or retain
/// output descriptors and must not occupy capacity while waiting for host PID1.
/// Linux /proc is the liveness authority; errors fall back to conservative kill(0).
pub(crate) fn process_group_live(group: i32) -> bool {
    assert!(group > 0);
    let scan = || -> io::Result<bool> { scan_process_group(group) };
    scan().unwrap_or_else(|_| {
        debug_assert!(group > 0);
        // SAFETY: signal zero probes the owned process group without modifying it.
        (unsafe { libc::kill(-group, 0) }) == 0 || io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
    })
}

/// Scans /proc for a live member of the owned process group.
fn scan_process_group(group: i32) -> io::Result<bool> {
    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        if !is_process_directory(&entry.file_name()) {
            continue;
        }
        let Some((process_group, terminal)) = read_proc_stat(&entry.path().join("stat"))? else {
            continue;
        };
        if process_group != group {
            continue;
        }
        if !terminal {
            debug_assert!(process_group == group);
            return Ok(true);
        }
        // A pthread-exited group leader may be a zombie with live threads.
        // Only ignore it when every remaining task is terminal as well.
        if group_has_live_task(group, &entry.path())? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Only numeric /proc entries describe processes.
fn is_process_directory(name: &std::ffi::OsStr) -> bool {
    name.as_bytes().iter().all(u8::is_ascii_digit)
}

/// Reports whether any task of the process still runs inside the group.
fn group_has_live_task(group: i32, process_path: &Path) -> io::Result<bool> {
    let tasks = match std::fs::read_dir(process_path.join("task")) {
        Ok(tasks) => tasks,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    for task in tasks {
        let Some((task_group, terminal)) = read_proc_stat(&task?.path().join("stat"))? else {
            continue;
        };
        if task_group == group && !terminal {
            debug_assert!(group > 0);
            return Ok(true);
        }
    }
    Ok(false)
}

fn stat_group_state(bytes: &[u8]) -> io::Result<(i32, bool)> {
    // comm may itself contain spaces and ')'; fields after its last ')' are
    // state, ppid, pgrp. Reject malformed snapshots rather than freeing early.
    let end = bytes
        .iter()
        .rposition(|byte| *byte == b')')
        .ok_or_else(|| io::Error::other("compile-slot-proc-stat"))?;
    let text = std::str::from_utf8(&bytes[end.saturating_add(1)..]).map_err(io::Error::other)?;
    let mut fields = text.split_whitespace();
    let state = fields.next().ok_or_else(|| io::Error::other("compile-slot-proc-state"))?;
    let _parent = fields.next().ok_or_else(|| io::Error::other("compile-slot-proc-parent"))?;
    let process_group = fields
        .next()
        .and_then(|value| value.parse::<i32>().ok())
        .ok_or_else(|| io::Error::other("compile-slot-proc-group"))?;
    Ok((process_group, matches!(state, "Z" | "X" | "x")))
}

fn read_proc_stat(path: &Path) -> io::Result<Option<(i32, bool)>> {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH) => {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let mut bytes = [0_u8; 4096];
    let mut count = 0;
    let mut is_complete = false;
    while !is_complete {
        debug_assert!(count <= bytes.len());
        if count == bytes.len() {
            return Err(io::Error::other("compile-slot-proc-stat-bound"));
        }
        match file.read(&mut bytes[count..]) {
            Ok(0) => {
                is_complete = true;
            }
            Ok(read) => count += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH) => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        }
    }
    debug_assert!(count <= bytes.len());
    if count == 0 {
        return Ok(None);
    }
    stat_group_state(&bytes[..count]).map(Some)
}

pub(crate) fn terminate_process_group(group: i32) {
    assert!(group > 0);
    // SAFETY: negative pid addresses only this child's fresh cooperative group.
    unsafe {
        libc::kill(-group, libc::SIGKILL);
    }
}

fn wait_process_group(group: i32) {
    while process_group_live(group) {
        std::thread::sleep(CHECK_INTERVAL);
    }
}

fn child_handshake(descriptor: RawFd) -> io::Result<()> {
    // SAFETY: setpgid creates this child's group before any compiler can exec;
    // inherited compiler descendants remain in it through driver failure.
    if unsafe { libc::setpgid(0, 0) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let mut message = [0_u8; 5];
    message[0] = START_CHILD;
    // SAFETY: getpid has no preconditions and is async-signal-safe.
    let pid = unsafe { libc::getpid() } as u32;
    message[1..].copy_from_slice(&pid.to_be_bytes());
    debug_assert_eq!(message[0], START_CHILD);
    let mut offset_bytes = 0;
    while offset_bytes < message.len() {
        // SAFETY: valid inherited socket and immutable remainder of stack buffer.
        let count = unsafe {
            libc::send(
                descriptor,
                message[offset_bytes..].as_ptr().cast(),
                message.len().saturating_sub(offset_bytes),
                libc::MSG_NOSIGNAL,
            )
        };
        if count < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if count == 0 {
            return Err(io::Error::from_raw_os_error(libc::EPIPE));
        }
        offset_bytes += count as usize;
    }
    debug_assert_eq!(offset_bytes, message.len());
    let mut ack = 0_u8;
    let mut is_acknowledged = false;
    while !is_acknowledged {
        // SAFETY: one writable stack byte and an inherited socket descriptor.
        let count = unsafe { libc::recv(descriptor, std::ptr::addr_of_mut!(ack).cast(), 1, 0) };
        if count == 1 && ack == CHILD_READY {
            is_acknowledged = true;
        } else if count < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        } else {
            return Err(io::Error::from_raw_os_error(libc::ECONNRESET));
        }
    }
    debug_assert_eq!(ack, CHILD_READY);
    Ok(())
}

/// Nonblocking connect avoids an unbounded wait on a full Unix accept backlog.
pub(crate) fn connect_bounded(path: &Path, timeout: Duration) -> io::Result<UnixStream> {
    let address = unix_socket_address(path)?;
    // SAFETY: socket arguments specify a Linux Unix stream with owned CLOEXEC fd.
    let raw = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, 0) };
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    debug_assert!(raw >= 0);
    // SAFETY: successful socket returned a fresh descriptor.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    connect_within_deadline(raw, &address, timeout)?;
    // SAFETY: clearing O_NONBLOCK on an owned socket preserves its CLOEXEC flag.
    if unsafe { libc::fcntl(raw, libc::F_SETFL, 0) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(UnixStream::from(descriptor))
}

/// Builds the Unix socket address for a filesystem path.
fn unix_socket_address(path: &Path) -> io::Result<libc::sockaddr_un> {
    let bytes = path.as_os_str().as_bytes();
    // SAFETY: sockaddr_un is plain data; zero initializes its pathname padding.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if bytes.is_empty() || bytes.len() >= address.sun_path.len() || bytes.contains(&0) {
        return Err(io::Error::from_raw_os_error(libc::EINVAL));
    }
    address.sun_family =
        libc::sa_family_t::try_from(libc::AF_UNIX).map_err(|_| io::Error::from_raw_os_error(libc::EINVAL))?;
    for (destination, byte) in address.sun_path.iter_mut().zip(bytes) {
        *destination = *byte as libc::c_char;
    }
    debug_assert!(!bytes.is_empty());
    Ok(address)
}

/// Connects without blocking on a saturated accept backlog.
fn connect_within_deadline(raw: libc::c_int, address: &libc::sockaddr_un, timeout: Duration) -> io::Result<()> {
    // SAFETY: initialized sockaddr_un is correctly sized and lives through call.
    let result = unsafe {
        libc::connect(raw, std::ptr::from_ref(address).cast(), std::mem::size_of_val(address) as libc::socklen_t)
    };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    // EAGAIN on a Unix socket means backlog saturation, not an in-flight
    // connection. Degrade immediately rather than polling a false success.
    if error.raw_os_error() != Some(libc::EINPROGRESS) {
        return Err(error);
    }
    await_connect_ready(raw, timeout)
}

/// Waits for a pending nonblocking connect to settle.
fn await_connect_ready(raw: libc::c_int, timeout: Duration) -> io::Result<()> {
    let mut pollfd = libc::pollfd {
        fd: raw,
        events: libc::POLLOUT,
        revents: 0,
    };
    // The poll timeout saturates at the i32 millisecond range and never waits less than a tick.
    let timeout_ms = i32::try_from(timeout.as_millis().max(1).min(i32::MAX as u128)).map_or(i32::MAX, |millis| millis);
    debug_assert!(timeout_ms >= 1);
    // SAFETY: one valid pollfd and a bounded millisecond timeout.
    let ready = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
    if ready == 0 {
        return Err(io::Error::from(io::ErrorKind::TimedOut));
    }
    if ready < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut pending_error: libc::c_int = 0;
    let mut length_bytes = std::mem::size_of_val(&pending_error) as libc::socklen_t;
    debug_assert!(length_bytes > 0);
    // SAFETY: correctly sized writable SO_ERROR buffers and valid fd.
    let status = unsafe {
        libc::getsockopt(
            raw,
            libc::SOL_SOCKET,
            libc::SO_ERROR,
            std::ptr::addr_of_mut!(pending_error).cast(),
            std::ptr::addr_of_mut!(length_bytes).cast(),
        )
    };
    if status < 0 {
        return Err(io::Error::last_os_error());
    }
    if pending_error != 0 {
        return Err(io::Error::from_raw_os_error(pending_error));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixListener;
    use std::sync::mpsc;
    use std::thread;

    use crunch_rust_cache_core::slots::SLOT_POLICY_SCHEMA;
    use crunch_rust_cache_core::slots::SlotFairness;

    use super::*;

    fn policy(total_slots: u32) -> SlotPolicy {
        SlotPolicy {
            schema: SLOT_POLICY_SCHEMA.to_string(),
            total_slots,
            per_build_max: 1,
            total_queue: 8,
            per_build_queue: 2,
            wait_timeout_ms: 3_000,
            fairness: SlotFairness::RoundRobin,
        }
    }

    fn fixture(
        root: &Path,
        policy: SlotPolicy,
        connections: u32,
    ) -> (Arc<SlotAuthority>, std::path::PathBuf, thread::JoinHandle<()>, mpsc::Receiver<SlotReceipt>) {
        let path = root.join("slots.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let (sink, receipts) = mpsc::channel();
        let authority = Arc::new(
            SlotAuthority::with_receipt_sink(
                policy,
                Arc::new(move |receipt| {
                    let _ = sink.send(receipt.clone());
                }),
            )
            .unwrap(),
        );
        let server_authority = Arc::clone(&authority);
        let server = thread::spawn(move || {
            let mut workers = Vec::new();
            for _ in 0..connections {
                let (mut stream, _) = listener.accept().unwrap();
                let authority = Arc::clone(&server_authority);
                workers.push(thread::spawn(move || {
                    let request = read_frame(&mut stream, SLOT_FRAME_BYTES).unwrap();
                    // A dead client legitimately makes the final receipt write fail.
                    let _ = authority.handle_connection(&mut stream, request);
                }));
            }
            for worker in workers {
                worker.join().unwrap();
            }
        });
        (authority, path, server, receipts)
    }

    fn granted(path: &Path, build: &str) -> SlotLease {
        match acquire_slot(path, build, Duration::from_secs(4)) {
            SlotClientOutcome::Granted(lease) => lease,
            SlotClientOutcome::Degraded(receipt) => panic!("unexpected degraded slot: {receipt:?}"),
        }
    }

    fn await_occupancy(authority: &SlotAuthority, expected: (u32, u32)) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while authority.occupancy().unwrap() != expected {
            assert!(Instant::now() < deadline, "occupancy did not reach {expected:?}");
            thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn simultaneous_build_children_respect_total_slots() {
        let root = tempfile::tempdir().unwrap();
        let (authority, socket, server, receipts) = fixture(root.path(), policy(2), 4);
        let (started_tx, started_rx) = mpsc::channel();
        let mut releases = Vec::new();
        let mut clients = Vec::new();
        for id in 0..4 {
            let socket = socket.clone();
            let started_tx = started_tx.clone();
            let (release_tx, release_rx) = mpsc::channel();
            releases.push(release_tx);
            clients.push(thread::spawn(move || {
                let mut lease = granted(&socket, &format!("build-{id}"));
                let mut command = Command::new("/bin/sleep");
                command.arg("30");
                let mut child = lease.spawn(&mut command).unwrap();
                started_tx.send(id).unwrap();
                release_rx.recv_timeout(Duration::from_secs(4)).unwrap();
                child.kill().unwrap();
                child.wait().unwrap();
                assert_eq!(lease.release().unwrap().decision, SlotDecision::Released);
            }));
        }
        let first = started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let second = started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        await_occupancy(&authority, (2, 2));
        assert!(matches!(started_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        releases[first].send(()).unwrap();
        let third = started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        releases[second].send(()).unwrap();
        let fourth = started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        releases[third].send(()).unwrap();
        releases[fourth].send(()).unwrap();
        for client in clients {
            client.join().unwrap();
        }
        server.join().unwrap();
        assert_eq!(authority.occupancy().unwrap(), (0, 0));
        let receipts: Vec<_> = receipts.try_iter().collect();
        assert_eq!(receipts.iter().filter(|r| r.decision == SlotDecision::Granted).count(), 4);
        assert_eq!(receipts.iter().filter(|r| r.decision == SlotDecision::Released).count(), 4);
        assert!(receipts.iter().all(|r| r.active_slots <= 2));
    }

    #[test]
    fn disconnected_driver_kills_owned_child_before_reclaiming() {
        let root = tempfile::tempdir().unwrap();
        let (authority, socket, server, receipts) = fixture(root.path(), policy(1), 2);
        let mut first = granted(&socket, "first");
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let mut child = first.spawn(&mut command).unwrap();
        drop(command);
        drop(first); // Same socket EOF observed after driver process death.
        let (next_tx, next_rx) = mpsc::channel();
        let next = thread::spawn(move || {
            next_tx.send(granted(&socket, "second")).unwrap();
        });
        let second = next_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGKILL));
        assert_eq!(second.release().unwrap().decision, SlotDecision::Released);
        next.join().unwrap();
        server.join().unwrap();
        assert_eq!(authority.occupancy().unwrap(), (0, 0));
        let receipts: Vec<_> = receipts.try_iter().collect();
        assert_eq!(
            receipts.iter().filter(|r| r.build_id == "first" && r.decision == SlotDecision::Released).count(),
            1
        );
    }

    #[test]
    fn successful_spawn_keeps_redirected_output_and_explicit_release() {
        let root = tempfile::tempdir().unwrap();
        let (authority, socket, server, _) = fixture(root.path(), policy(1), 1);
        let mut lease = granted(&socket, "stdio");
        let output = tempfile::tempfile().unwrap();
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf exact-output; printf exact-error >&2"]);
        command.stdout(output.try_clone().unwrap());
        command.stderr(output.try_clone().unwrap());
        let mut child = lease.spawn(&mut command).unwrap();
        assert!(child.wait().unwrap().success());
        use std::io::Seek;
        use std::io::SeekFrom;
        let mut output = output;
        output.seek(SeekFrom::Start(0)).unwrap();
        let mut text = String::new();
        output.read_to_string(&mut text).unwrap();
        assert_eq!(text, "exact-outputexact-error");
        assert_eq!(authority.occupancy().unwrap(), (1, 0));
        assert_eq!(lease.release().unwrap().decision, SlotDecision::Released);
        server.join().unwrap();
        assert_eq!(authority.occupancy().unwrap(), (0, 0));
    }

    #[test]
    fn absent_daemon_degrades_without_fabricated_grant_or_wait() {
        let root = tempfile::tempdir().unwrap();
        let start = Instant::now();
        let outcome = acquire_slot(&root.path().join("absent.sock"), "offline", Duration::from_secs(3));
        match outcome {
            SlotClientOutcome::Degraded(receipt) => {
                assert!(matches!(receipt.reason, SlotDegradedReason::AuthorityUnavailable));
                assert!(receipt.authority_receipt.is_none());
                assert_eq!(receipt.disposition, "degraded");
            }
            SlotClientOutcome::Granted(_) => panic!("absent authority fabricated a grant"),
        }
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn occupied_authority_times_out_and_removes_waiter() {
        let root = tempfile::tempdir().unwrap();
        let mut p = policy(1);
        p.wait_timeout_ms = 100;
        let (authority, socket, server, _) = fixture(root.path(), p, 2);
        let first = granted(&socket, "first");
        // Keep the first pre-spawn reservation alive past the second wait bound
        // by starting a real child, whose lifetime cannot expire with the queue.
        let mut first = first;
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let mut child = first.spawn(&mut command).unwrap();
        match acquire_slot(&socket, "second", Duration::from_secs(1)) {
            SlotClientOutcome::Degraded(receipt) => {
                assert!(matches!(receipt.reason, SlotDegradedReason::AuthorityRejected));
                assert_eq!(receipt.authority_receipt.unwrap().reason, Some(SlotRejectReason::WaitTimeout));
            }
            SlotClientOutcome::Granted(_) => panic!("occupied authority granted above bound"),
        }
        assert_eq!(authority.occupancy().unwrap(), (1, 0));
        child.kill().unwrap();
        child.wait().unwrap();
        first.release().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn process_group_member_holds_slot_after_leader_exit() {
        let root = tempfile::tempdir().unwrap();
        let (authority, socket, server, _) = fixture(root.path(), policy(1), 2);
        let mut lease = granted(&socket, "leader");
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let mut leader = lease.spawn(&mut command).unwrap();
        // Use an independently reapable member in the same group, exercising
        // descendant-group retention without relying on the host PID1 reaper.
        let mut member_command = Command::new("/bin/sleep");
        member_command.arg("30").process_group(leader.id() as i32);
        let mut member = member_command.spawn().unwrap();
        leader.kill().unwrap();
        leader.wait().unwrap();
        let release = thread::spawn(move || lease.release().unwrap());
        let (next_tx, next_rx) = mpsc::channel();
        let next = thread::spawn(move || {
            next_tx.send(granted(&socket, "waiting")).unwrap();
        });
        await_occupancy(&authority, (1, 1));
        assert!(member.try_wait().unwrap().is_none());
        assert!(matches!(next_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        member.kill().unwrap();
        assert_eq!(
            next_rx.recv_timeout(Duration::from_secs(3)).unwrap().release().unwrap().decision,
            SlotDecision::Released
        );
        // The member has deliberately not been reaped. Its zombie must not
        // retain capacity, even on hosts whose PID1 does not reap orphans.
        member.wait().unwrap();
        assert_eq!(release.join().unwrap().decision, SlotDecision::Released);
        next.join().unwrap();
        server.join().unwrap();
        assert_eq!(authority.occupancy().unwrap(), (0, 0));
    }
}
