## Phase 1: Release the FUSE channels

- [x] [serial] T1.1 Wake every session channel after a successful `umount()` in `FuseDaemon::unmount`, before `wait()`, keeping the existing error mapping and `#[instrument]` attributes and naming the reason in a comment. r[vendored_snix.fuse_unmount_release]
  - Evidence: `vendor/snix-castore/src/fs/fuse/mod.rs` diff.
- [x] [serial] T1.2 Add `unmount_with_held_descriptor` to the vendored FUSE tests: hold a file descriptor opened inside the mount across the unmount, require the unmount within a named bound, require the held read to fail with `ENOTCONN`, and keep the timeout branch from hanging. r[vendored_snix.fuse_unmount_release]
  - Evidence: the new test passing with the wake and its timeout branch under the temporarily removed wake.
- [x] [serial] T1.3 Record the local vendored modification and its upstream-sync obligation in ADR 0170, and index it in `adr/README.md`. r[vendored_snix.fuse_unmount_release]
  - Evidence: `adr/0170-*.md` and the `adr/README.md` row.

## Phase 2: Checks and records

- [x] [serial] T2.1 Run the new and existing vendored FUSE tests, rustfmt on both touched files, the route and foreign-import CLI suites, and `cargo build -p mantle --bin mantle`; record the exact transcripts in `evidence/` together with the diagnosis README and its two sample files. r[vendored_snix.fuse_unmount_release]
  - Evidence: `evidence/checks.md` and the `evidence/` files it indexes.
- [x] [serial] T2.2 Run `cairn validate`, the proposal/design/tasks gates, and `git diff --check`; commit the change. r[vendored_snix.fuse_unmount_release]
  - Evidence: gate transcripts and the commit.
