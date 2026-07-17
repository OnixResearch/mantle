## Implementation

- [x] [serial] I1 Regenerate the checkout-local ignored Cargo directory source from the locked graph and prove offline resolution with an empty Cargo home. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: pueue task `73` produced a fresh `cargo vendor --locked` tree including Cargo-authored `cap-fs-ext`, `cap-primitives`, and `cap-std`; tasks `78` and `83` passed locked offline metadata, with task `83` using an empty `CARGO_HOME` and `CARGO_NET_OFFLINE=true`.
- [x] [serial] I2 Replace proof-toolchain-incompatible UTF-8 width usage with a stable semantic equivalent and run focused positive/negative diagnostic tests. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: task `81` exposed the pinned-nightly compile failure; task `89` passed Rustfmt and all 10 focused remote-failure-debug tests after the stable expression repair.
- [x] [serial] I3 Add the tracked `config/` policy payload to the fixed staged-source closure and prove it is copied without widening excluded roots. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: task `97` exposed the missing compile-time policy path; task `22` passed the positive allowlist copy test and both staged-path policy tests while retaining exclusions for `target/`, arbitrary scratch files, private `.pi` content, parent traversal, and empty paths.
- [x] [serial] I4 Preserve atomic no-replace publication across glibc and bootstrap musl targets through one syscall-backed Linux shell with positive and negative race coverage. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: task `24` exposed the missing musl `libc::renameat2` binding at all four production call sites; task `26` passed the new direct success/existing-destination tests plus OCI, release-publication, attempt-log, and remote-failure no-clobber fixtures.
- [x] [serial] I5 Replace the production compile-time checkout lookup with the source-or-embedded Nickel stdlib resolver and retain positive/negative typed-config coverage. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: task `30` compiled the complete staged release binary and then failed the unchanged path-leak scan on runtime data containing `/tmp/build/crunch`; task `32` passed both positive and negative typed Nickel dispatcher contract tests after routing production lookup through `crunch_eval::stdlib::stdlib_import_path()`.

## Validation and lifecycle

- [x] [serial] V1 Record the failing vendor, compiler, FUSE, staged-source, target-libc, and path-leak baselines without promoting partial progress into success. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: `evidence/baseline.md` records tasks `67`, `81`, `92`, `97`, `24`, and `30` with each exact frontier and claim boundary.
- [x] [serial] V2 Run the actual self-build vendor validator plus focused positive/negative source-staging tests against the repaired checkout. r[bootstrap_inventory.self_build_source_closure]
  - Evidence: tasks `92` and `97` passed source staging and the actual lock/package/file checksum validator before later host/source frontiers; task `22` passed 1 allowlist copy test, 2 staged-path policy tests, and all 10 vendor-validator positive/negative unit tests.
- [ ] [serial] V3 Run the canonical fixed-point proof with strict later-stage hermeticity and inspect stage1/stage2 and bootstrap-tool identity evidence. r[bootstrap_inventory.self_build_source_closure]
- [ ] [serial] V4 Run focused/full quality, dependency audit, Cairn validation and gates, sync and inspect the accepted requirement, add evidence-backed Tracey links, and archive only after exact post-archive evidence is durable. r[bootstrap_inventory.self_build_source_closure]
