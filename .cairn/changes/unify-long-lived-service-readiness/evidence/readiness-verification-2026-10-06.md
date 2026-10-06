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
