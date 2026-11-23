# Design: Release the castore FUSE inputs promptly after a sandbox build

## Context

`vendor/snix-castore/src/fs/fuse/mod.rs` starts `FuseDaemon` as `num_threads` threads, each looping
over `FuseChannel::get_request()` and handling one FUSE request at a time. `unmount()` sends
`FuseSession::umount()`, which is a lazy unmount on Linux
(`vendor/fuse-backend-rs/src/transport/fusedev/linux_session.rs`): `umount2(MNT_DETACH)`, or
`fusermount3 --unmount --lazy`. A lazy unmount detaches the mount from the namespace but keeps the
superblock and the FUSE connection alive while any reference remains. The server threads poll their
`/dev/fuse` descriptors, so they do not exit until the kernel aborts the connection, which requires
every reference to be gone. `unmount()` then calls `wait()`, which joins them.

Production realizes the inputs filesystem with `allow_other = false` and drops the guard when the
build ends. During the diagnosed stall no live process Mantle can read held the mount, a descriptor
on it, a cwd, root, or mapping; the retaining reference was a mount copy in a namespace no process
occupies. Specifically, the running Nix 2.36.0 daemon keeps every sandboxed build's
pre-`pivot_root` mount namespace open as `sandboxMountNamespace`, private, until
`unprepareBuild()` runs at the end of that build
(`src/libstore/linux/build/linux-derivation-builder.cc` at tag 2.36.0). Our lazy unmount does not
propagate into that private copy, so an unrelated Nix build that started while the inputs mount
existed kept the connection alive; both stalls ended seconds after such a build finished. Mantle
cannot control those references, so it must not wait for them.

`fuse-backend-rs` already exposes the missing half: `FuseSession::wake()` writes to each channel's
eventfd, so `get_request()` returns `None`, the thread's `start()` loop ends, and its `/dev/fuse`
descriptor closes.

## Decisions

### Decision: wake the channels after a successful unmount, then join

**Choice:** in `FuseDaemon::unmount`, keep the existing `umount()` call and its error mapping, and
on success call `self.session.lock().wake()`, mapping a wake failure to `io::Error::other` the same
way. `wait()` follows unchanged. A comment states why the wake is required: the unmount is lazy, so
the kernel would otherwise abort the connection only when the last reference disappears, and the
build must not wait for references it does not own.

**Rationale:** closing the channel descriptors aborts the FUSE connection immediately. A stale
reference that reads the detached filesystem then gets `ENOTCONN` instead of blocking, and the join
returns. It is the smallest change that removes the dependency on an outside reference, and it uses
the transport's own shutdown primitive rather than a timeout or a detached thread.

**Rejected alternatives:** unmounting non-lazily (still waits for references, and fails when they
exist); abandoning the join (`FuseDaemon` would leak its thread pool and the mount's connection);
adding a join timeout (the threads would still be blocked on the descriptor, and the daemon would
report success while its descriptors stay open); changing `snix-build`'s teardown order (the inputs
guard must drop before the sandbox result is collected, and the fault is in the daemon's own
shutdown path).

**Consequence:** an unmounted filesystem no longer serves requests, by design. A late access fails
rather than succeeding against a detached mount; the build has already finished, so nothing Mantle
owns reads it afterwards.

### Decision: pin the behavior with a test that holds a reference across the unmount

**Choice:** add `vendor/snix-castore/src/fs/fuse/tests.rs::unmount_with_held_descriptor`. It mounts
a filesystem containing `BLOB_A`, opens that file with `std::fs::File::open`, keeps the descriptor,
and clones the `FuseDaemon` into a thread that calls `unmount()`. The test waits on a channel
`recv_timeout(UNMOUNT_RETURN_TIMEOUT)` (10 s, named constant) and requires the unmount to report
success within that bound. It then reads through the held descriptor and requires an error of kind
`NotConnected` (`ENOTCONN`), closes the descriptor, and joins the unmount thread. If the wait times
out, the test drops the held descriptor first so the old-code thread can finish, joins it, and then
fails with a message naming the bound; the test can therefore not hang.

**Rationale:** the held descriptor reproduces the production condition (a reference that keeps the
lazily detached filesystem alive) without depending on an outside process. The bound is generous
compared with the milliseconds a wake takes, so it does not mask slowness on a loaded host, and it
turns the pre-fix behavior into a failure rather than a hung run. Checking `ENOTCONN` also pins the
observable consequence the change claims, not only that a join returned.

**Rejected alternatives:** a test that only mounts and unmounts (passes before and after the fix);
asserting the elapsed time against the 7.5-minute production stall (slow and environment-specific);
leaving the descriptor open at the end (the FUSE connection would stay alive and later tests could
interfere).

### Decision: record the vendored modification in ADR 0170

**Choice:** ADR 0170 states the local modification, the upstream file it touches, the reason, the
test that guards it, and the requirement that an upstream sync keeps it (the convention of ADR 0163
for vendored `nix-compat`/`snix-build`). `adr/README.md` gains its row.

**Rationale:** `vendor/` is an edited snapshot, not a live fork; without a record the next upstream
sync silently reintroduces the stall.

## Risks / Trade-offs

- `wake()` must happen while the session's uppers are still alive; it is called before `wait()`, and
  the mutex is released before joining, so a channel thread cannot deadlock on it.
- If `umount()` fails, no wake happens and no join happens — unchanged from today, so a failed
  unmount still surfaces its error.
- A future upstream `unmount` may add the wake itself; the ADR requires the sync to keep the
  behavior and permits dropping the local line once the vendored code guarantees it.

## Evidence

- `fs::fuse::tests::unmount_with_held_descriptor` passes with the wake and fails (its timeout
  branch) with the wake removed; the failing run is recorded under `evidence/`.
- The existing `fs::fuse::tests::` suite passes unchanged.
- `tests/cache_inputs_build_route.rs` and `tests/foreign_import_cli.rs` pass unchanged; the route a
  realization uses is not touched.
