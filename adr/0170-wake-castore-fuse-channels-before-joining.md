# 0170. Wake castore FUSE channels before joining the inputs daemon

Status: Accepted

## Context

A realization materializes a sandbox's inputs as a castore FUSE mount and drops its guard when the
build ends. On 2026-11-22 the Octet Aeneas run on route `cache-inputs-build-roots-v2` then stalled
for about 7.5 minutes after the sandbox exited: one tokio worker sat in
`snix_build::bwrap::Bwrap::run_inner`, dropping the inputs guard, through `FuseDaemon::drop` into
`FuseDaemon::unmount` → `threadpool::ThreadPool::join`, while the four FUSE server threads waited in
`FuseChannel::get_request` → `epoll_wait`.

`vendor/snix-castore/src/fs/fuse/mod.rs` unmounts lazily (`umount2(MNT_DETACH)`, or
`fusermount3 --unmount --lazy` in `vendor/fuse-backend-rs/src/transport/fusedev/linux_session.rs`)
and then joins without waking the channels. A lazy unmount detaches the mount but keeps the
superblock and its FUSE connection while any reference remains, and the server threads leave only
when the kernel aborts that connection. The last reference belonged to something outside Mantle:
the running Nix 2.36.0 daemon keeps each sandboxed build's pre-`pivot_root` mount namespace open as
`sandboxMountNamespace`, a private recursive copy of the host mount tree that our lazy unmount does
not reach and that only closes in `unprepareBuild()` when that build ends. A concurrent Nix build
started while the inputs mount existed therefore held it, and both diagnosed stalls ended within
seconds of such a build finishing. The diagnosis, including the Nix source references, is in
`.cairn/changes/release-fuse-inputs-promptly/evidence/` (`stall-diag-README.md`,
`stall-2-eu-stack.txt`, `conn78.txt`).

`fuse-backend-rs` already exposes `FuseSession::wake()`, which signals each channel's eventfd;
`get_request` then returns `None`, the thread's loop ends, and its `/dev/fuse` descriptor closes.

## Decision Drivers

- A build must not wait for a reference it does not own, and a stalled realization has no bounded
  failure mode.
- The inputs mount must still be unusable after teardown; a stale reference should observe a
  disconnected filesystem rather than receive data from a detached mount.
- The change stays inside the shutdown path of the vendored daemon: no `snix-build` teardown
  reordering, no bwrap argument change, no other provider.
- The vendored tree is an edited snapshot, so an upstream sync must not silently reintroduce the
  stall.

## Decision

`FuseDaemon::unmount` keeps its existing `umount()` call and error mapping, and after a successful
unmount calls `session.wake()` before `wait()`. A wake failure is returned as `io::Error::other`,
like an unmount failure. The existing `#[instrument]` attributes stay. A comment states why the
wake is required: the unmount is lazy, the kernel would otherwise abort the connection only when the
last reference disappears, and the caller must not wait for that.

This is a Mantle-local modification of vendored `snix-castore`, recorded here so upstream syncs keep
it (the convention ADR 0163 states for vendored `nix-compat` and `snix-build`). An upstream
`FuseDaemon::unmount` that performs the wake itself supersedes this line; otherwise the local call
must survive the sync.

`vendor/snix-castore/src/fs/fuse/tests.rs::unmount_with_held_descriptor` guards the behavior: it
mounts a filesystem with a file, opens and holds that file's descriptor, unmounts on another thread
and requires the unmount to return within `UNMOUNT_RETURN_TIMEOUT` (10 s). After every daemon handle
is dropped, it reads through the held descriptor on a helper thread and requires `ENOTCONN` within
the same bound; a timeout drops the held descriptor before joining so the run cannot hang. The test
does not read the file before teardown, so the page cache cannot satisfy the final read.

## Consequences

- An unmounted castore filesystem no longer serves requests: a late access fails with `ENOTCONN`
  instead of delaying the realization. Nothing Mantle owns reads the inputs mount after the build.
- A failed unmount still returns its error and does not wake or join, as before.
- The vendored file carries a small local diff that an upstream sync must retain or replace with
  equivalent upstream behavior.
- The test needs `/dev/fuse` and the setuid `fusermount3` on `PATH`, like the rest of the vendored
  FUSE suite, and is outside the first-party gates.
- This does not claim that every detached-mount reference disappears, nor that the outside holder is
  identified. It claims only that Mantle stops waiting for it.
