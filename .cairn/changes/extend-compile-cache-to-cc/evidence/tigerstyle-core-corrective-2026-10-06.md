# CC cache core Tiger Style corrective, 2026-10-06

Scope: clean linked worktree at committed `851c101d000fa9c20d04cd786c1142cf743d7eef`; only `crates/crunch-rust-cache-core/src/cc.rs` changed. The dirty primary checkout was not edited. The pre-correction Nix log at `/nix/store/8vm63isbm97r23p5q4sivkgv2rp6zbj7-tigerstyle-consumer-check.drv` contains 21 diagnostics in CC core, including an unchecked root-index conversion and unsigned predecessor subtraction, and separately about 20 `crunch-service-readiness-core` diagnostics assigned to another owner. The failed pre-correction check was not rerun.

The correction bounds the JSON/base64 sizes with fixed-width `u64`, checks root counts into `u32` before comparing with the dependency's `u32` index, decomposes admission guards, uses safe predecessor access and one borrowed argument prefix for slash recognition, and asserts invariants only after validation. No Tiger Style lint was suppressed. Added focused regressions for last-valid/first-invalid root indexes and non-representable root counts, plus relative UTF-8 slash acceptance and absolute argument rejection. Existing dependency order and wire/object admission behavior remain covered by the core suite.

Observed verification:

- Before edit: `cargo test --offline -p crunch-rust-cache-core cc::tests:: --lib` using the installed nightly and private CC-only target: **10 passed**. The first bare `cargo` attempt failed because it was absent from `PATH`; the first `nix develop` attempt was stopped when it tried automatic garbage collection, and an offline, GC-disabled devshell attempt timed out before tests ran.
- After edit: the same focused core command: **12 passed**, 25 filtered. Final `cargo test --offline -p crunch-rust-cache-core --lib`: **37 passed**, zero failed.
- Final pinned Nix toolchain `cargo clippy --offline -p crunch-rust-cache-core --lib -- -D warnings`: **exit 0**. An earlier Clippy invocation detected one `unnecessary_map_or`; corrected it and the final invocation passed.
- Final nightly leaf `rustfmt --edition 2024 --config skip_children=true --check crates/crunch-rust-cache-core/src/cc.rs` and `git diff --check`: **exit 0**.
- Offline `nix --option min-free 0 --option max-free 0 --option eval-cache false build --offline --no-link .#checks.x86_64-linux.fmt`: **exit 0** after the principal correction. Subsequent small Rust edits passed the final leaf rustfmt check but the whole Nix fmt derivation was not repeated.

Limits and blockers:

- The post-correction Nix Tiger Style derivation `/nix/store/6dky13frhhcac9yw7rdirgbfl94r2fz9-tigerstyle-consumer-check.drv` was started then **cancelled without a result** as shared free space approached the required 12 GiB floor. Thus the reported 21 CC lint diagnostics are addressed in source but not verified cleared by the full Nix gate; the independent service-readiness diagnostics also remain outside this commit. An attempted `nix run .#tigerstyle -- check -- -p crunch-rust-cache-core --lib` failed while compiling an unrelated workspace `zstd-sys` dependency in the shared Cargo target (permission denied) before CC diagnostics.
- Post-correction real GCC/G++ cache-on/off object and depfile parity is **unverified**. The focused `crunch-cc-driver` integration test could not run: the private-target Cargo build stopped in external pinned `casita` source `nar.rs:88` with `GenericArray` lacking `as_bytes` (`E0599`). No Casita or vendor source was edited, and the same unpatched command was not retried. Earlier source-bound driver parity is not a postfix proof.
- No StageX, fixed-point, T1.1 origin, archive, or release claim follows from these scoped observations.
