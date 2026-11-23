# Change: Release the castore FUSE inputs promptly after a sandbox build

## Why

After a sandbox build finishes, a Mantle realization can stall for minutes before it ingests the
output. In the 2026-11-22 Octet Aeneas run on route `cache-inputs-build-roots-v2` the stall was
about 7.5 minutes; the Terrane WP8 run showed 42 s.

The diagnosis (`/home/brittonr/wt/convergence-evidence/followups/stall-diag/README.md`, copied into
`evidence/`) shows where the realization waits after the sandbox exits:

- one tokio worker in `snix_build::bwrap::Bwrap::run_inner`, dropping the inputs guard →
  `FuseDaemon::drop` → `FuseDaemon::unmount` → `threadpool::ThreadPool::join`;
- four `fuse_server` threads in `FuseChannel::get_request` → `epoll_wait`.

`FuseDaemon::unmount` (`vendor/snix-castore/src/fs/fuse/mod.rs`) lazy-unmounts the inputs
filesystem (`umount2(MNT_DETACH)` / `fusermount3 --lazy`) and then joins the server threads without
waking them. Those threads leave `epoll_wait` only when the kernel aborts the FUSE connection, and
the kernel does that only once the lazily detached filesystem has no remaining reference. Here the
last reference was a private mount copy held by the Nix daemon's saved sandbox namespace for a
concurrent build, so the realization waited for a reference it does not own and cannot see.

## What Changes

- In `FuseDaemon::unmount`, after `umount()` succeeds, call the session's `wake()` before `wait()`.
  Each channel thread leaves `get_request`, closes its `/dev/fuse` descriptor, and the connection
  aborts. A stale reference then observes a disconnected filesystem (`ENOTCONN`) instead of
  delaying the realization. Error propagation and the `#[instrument]` attributes stay as they are,
  and a short comment names the reason.
- Add a vendored snix-castore FUSE regression test: mount a filesystem with a file, open the file
  and hold the descriptor, unmount on another thread, and require the unmount to return within a
  named bound; the held descriptor's next read must fail, after which the descriptor is closed.
- Record the local vendored modification in ADR 0170 and index it, so upstream syncs keep it.

## Impact

- **Files**: `vendor/snix-castore/src/fs/fuse/mod.rs`,
  `vendor/snix-castore/src/fs/fuse/tests.rs`, `adr/0170-*.md`, `adr/README.md`, and this change's
  Cairn artifacts.
- **Testing**: `cargo test -p snix-castore --features fs --lib fs::fuse::tests::` (new test plus the
  existing FUSE tests, with `/dev/fuse` and the setuid `fusermount3`), rustfmt on both touched
  vendored files, `cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli`,
  and `cargo build -p mantle --bin mantle`.
- **Non-goals**: changing `snix-build` teardown ordering, the bwrap arguments, any other provider,
  or the upstream `fuse-backend-rs` transport. The change does not promise that every detached
  mount reference disappears; it stops Mantle from waiting on references it does not own.
