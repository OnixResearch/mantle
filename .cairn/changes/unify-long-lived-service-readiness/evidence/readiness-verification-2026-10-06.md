# Readiness verification, 2026-10-06

Scope: the isolated primary checkout at `/home/brittonr/.cargo-target/mantle-rust-script-pin-20261004/source`, with a private `readiness-final-target` and private temporary directory for final Rust runs. These are bounded readiness observations, not a self-hosting, protected StageX, native Cairn archive, or release proof. The T1.1 current-`origin/main` provenance and blocked-dependency baseline remain open. This proposed Cairn change is not archived.

## Exercised paths

The readiness worker's captured test results were:

| Path | Exact result | Captured run |
| --- | --- | --- |
| Pure readiness core, including distinct simultaneous IDs, blocked/reblocked dependencies, malformed ingress, early exits, and all four policy decisions | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` | `artifact://32383` |
| Signed-plan stage dependency graph and dependency loss | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2546 filtered out` | `artifact://32644` |
| Existing doctor JSON failure result | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out` | `artifact://32723` |
| Authenticated remote stdio ticket/request | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out` | `artifact://32795` |
| Rust wrapper cold result and local cache reuse | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out` | `artifact://32970` |

The worker additionally checked the pure core on wasm and ran private CLI builds. Live coordination subscribers observed separate `started`/`ready` IDs, `on-error` declaration, terminal state and three retractions; a default-Off valid C frame was denied without a `ready` assertion. Authenticated remote serve and the standalone daemon exercised real requests, while a rejected remote ticket and safe proof preflight reported `started`/`failed`, never `ready`. Doctor human/JSON output was byte-identical against absent, live, and nonreading sockets; the nonreading case added 90.156 ms. The worker's real GCC comparison found native, cold-cache, and cache-hit object files byte-identical (1,216 bytes each), and wrapped depfiles byte-identical (410 bytes each), under the frozen v2 driver/policy digest; stdout/stderr were empty. These observed outputs do not establish general compiler correctness or receipt/evidence authority from volatile coordination facts.

## Scope incidents and non-claims

Before the isolated-checkout directive, the previous worker transiently edited the user's original `src/remote_build.rs` and restored it by a hashline snapshot; there was no pre-edit whole-file hash, mode, or staged snapshot, so byte-for-byte pristine restoration is **not established**. Before the private-target directive the worker changed modes of four generated headers in the **global shared** Cargo target. The user-authorized inode-guarded mode-only remedy restored `zstd.h`, `zstd_errors.h`, and `zdict.h` from `0644` to their recorded prechange `0444`. At remediation `bzlib.h` was `0600` although its recorded initial mode was `0444`; without an initial inode or hash, the worker left it unchanged, and no original byte-identity claim is possible. Neither the original checkout nor the shared target is an authorized location for this continuation's edits.

Readiness does not admit builds, C cache entries, protected proof, signed plan output, or releases. T1.1 origin-main provenance, remaining native Cairn acceptance/archival, and broader integration/audit remain open; do not mark this proposed change completed on the strength of these scoped runs.

## Isolated Tiger Style corrective, 2026-10-06

Source base: `851c101d000fa9c20d04cd786c1142cf743d7eef`; only `crates/crunch-service-readiness-core/src/lib.rs` changed in this corrective worktree. The previously failed Nix derivation `/nix/store/8vm63isbm97r23p5q4sivkgv2rp6zbj7-tigerstyle-consumer-check.drv` reported 20 readiness-core errors (unchecked byte-count arithmetic, boolean names, graph recursion, production `expect`, and unreserved invalid-subject growth), alongside 21 separate `crunch-rust-cache-core/src/cc.rs` errors outside this branch.

The corrective uses saturating byte-count arithmetic so excess payload size still rejects, a bounded iterative graph traversal and dependency-ready check, typed missing-runtime errors instead of panics, and an exact maximum-capacity reservation for invalid subjects. A boundary test exercises both the empty graph and a chain of 64 services at the declared maximum. Focused baseline `cargo test --offline -p crunch-service-readiness-core --lib -- --test-threads=1`: `10 passed; 0 failed` (`artifact://34665`); after the corrective: `11 passed; 0 failed` (`artifact://34754`). `cargo fmt -p crunch-service-readiness-core --check` passed, and `cargo clippy --offline -p crunch-service-readiness-core --all-targets --no-deps -- -D warnings` passed with the private cached target. One *intermediate* scoped `cargo-tigerstyle check -p crunch-service-readiness-core` invocation completed warning-only (`10 warnings; 0 errors`) before the final nonrecursive dependency-ready traversal; it does **not** attest the final code.

No final Nix Tiger Style derivation or post-final-change Tiger Style lint was attempted: the user suspended heavyweight gates as available space approached the 12 GiB minimum. The independent `cc.rs` gate errors remain outside this corrective, and neither their remediation nor a combined Tiger Style pass is claimed.

## Combined-source readiness lint continuation, 2026-10-06

From committed source base `e257faabc9b32ea70ad0835cc9ee55daa9e14440`, the already-failed `/nix/store/w1imk9yf97636niv08fv4y3nid9hb17a-tigerstyle-consumer-check.drv` log identified four readiness-core findings: missing local assertions in declaration validation, active dependency availability, and bounded graph validation, plus nonpredicate `made_progress`. The same log identified 13 separate `crunch-live-state-core` errors, owned by a different corrective branch. This continuation adds checked local invariants to those three readiness functions and names the graph progress predicate `has_progress`; it does not alter the accepted readiness vocabulary or graph traversal. A negative boundary test rejects dependency and custom-state lists just beyond their caps.

In this e257-derived isolated worktree, focused `cargo test --offline -p crunch-service-readiness-core --lib -- --test-threads=1` completed `12 passed; 0 failed` (`artifact://35008`): existing positive empty/max-depth graph, ready/dependency recovery and maximum-size inputs, plus negative cycles, stale readiness, and both new over-limit lists. Scoped `cargo-tigerstyle check -p crunch-service-readiness-core` completed **warning-only** with `0 errors; 8 warnings` (one file-length and seven non-trait-import warnings, no assertion-density or boolean-name finding). `cargo clippy --offline -p crunch-service-readiness-core --all-targets --no-deps -- -D warnings` and `cargo fmt -p crunch-service-readiness-core --check` passed with a private target. These scoped observations are not a combined Nix Tiger Style pass; no full Nix consumer rebuild ran while the separate live-state findings remain to be integrated.
