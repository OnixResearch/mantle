# Focused validation evidence — 2026-09-09

Environment: direct nightly toolchain (`~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu`),
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=cc` (nix develop is blocked in this
worktree by crates.io 403 on Nix fetchurl; see drain-progress.md),
`CARGO_TARGET_DIR=/tmp/mantle-drain-target`.

## V2 focused tests (final run after all implementation commits)

```
$ cargo test -p crunch-store
test result: ok. 357 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (authority_source_policy)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (doc-tests incl. compile_fail capability fixtures)

$ cargo test --bin mantle remote_transfer::
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 2395 filtered out

$ cargo test --bin mantle remote_build
test result: ok. 153 passed; 0 failed; 0 ignored; 0 measured; 2243 filtered out

$ cargo test -p crunch-build --lib publication
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 675 filtered out
```

## V4 gate legs run

```
$ cargo fmt --check -p crunch-store -p crunch-pipeline   → pass (after cargo fmt)
$ git diff --check                                        → clean
$ cargo clippy -p crunch-store -p crunch-pipeline --no-deps --all-targets -- -D warnings
  → 0 errors (one pre-existing vendored-dep finding class excluded via --no-deps per repo policy)
$ cargo clippy --bin mantle --no-deps -- -D warnings
  → only pre-existing finding: src/remote_nominal.rs:55 `as_str` never used
    (file untouched by this change; baseline debt recorded, not a regression)
$ cargo test -p crunch-store                              → 357 + 2 + 9 passed
```

## V3 architecture checker (final)

```
$ cargo -q -Zscript tools/check_store_capability_boundary.rs --self-test
self-test: ok
$ cargo -q -Zscript tools/check_store_capability_boundary.rs --root .
raw_service_escape_count=0
writable_authority_escape_count=0
handle_construction_escape_count=0
```
(full report in architecture-validation.md)

## Blocked V4 leg

`nix flake check -L` is blocked in this worktree: crates.io returns HTTP 403 to
Nix's fetchurl for new vendored crate tarballs (cap-fs-ext 3.4.2 et al.), so the
worktree flake cannot realize its dev/vendor inputs. Resolution needed (UA or
mirror for the Nix fetcher, or a warmed store) before the archive gate can run.
