# Complete example projects

These directories combine source/configuration, package selectors, checks, and project-local runbooks. Run each command from the named project directory.

| Project | Primary command | Check or workflow | Capability |
|---|---|---|---|
| `generated-site/` | `mantle build` | `mantle build .#checks.site-content` | local + fast |
| `codegen-pipeline/` | `mantle build` | `mantle build .#checks.app` | local + fast |
| `c-library-cli/` | `mantle build` | `mantle build .#checks.test-greet` | heavy + first-build network |
| `rust-workspace/` | `mantle build` | `mantle build .#checks.smoke` | heavy + first-build network |
| `fetched-and-patched/` | `mantle build` | `mantle build .#checks.patch` | real network |
| `multi-output-sdk/` | `mantle build` | `mantle build .#checks.development` | heavy + first-build network |
| `schema-codegen/` | `mantle build` | `mantle build .#checks.integration` | heavy + first-build network |
| `reproducible-release/` | `mantle build` | `mantle build .#checks.reproducible` | local + fast |
| `signed-cache-roundtrip/` | `mantle build .#payload` | signed publication and fresh-store substitution runbook | heavy + loopback HTTP |
| `locked-dependency-lifecycle/` | `mantle check` | offline stale detection, selected refresh, and upgrade | local + fast |
| `cross-compiled-host-tool/` | `mantle build .#target` | `mantle build .#role-mismatch` must fail | heavy + first-build network |
| `store-gc-lifecycle/` | `mantle build .#retained` | persistent root, dry-run, GC, and mutation-lock runbook | heavy + local state |
| `delta-substitution/` | `mantle build .#source` | `cargo run -p mantle --example delta_substitution` | Rust adaptor |
| `release-witness-handoff/` | `mantle build .#source` | `cargo run -p mantle --example release_witness_handoff` | Rust adaptor |

The C, Rust, and SDK projects expose source or generated-source selectors that can be built without realizing every downstream package. Their compiled packages use Mantle-owned bootstrap/toolchain inputs rather than ambient host compilers or Cargo caches.

The fetched project pins both its unpacked upstream source and local patch. The reproducible release project builds independently named normalized archives, records the expected BLAKE3 digest, and includes a tamper-detection check.

The workflow tier adds signed cache exchange, lock refresh, host/target role separation, persistent garbage collection, delta transfer policy, and signed witness quorum. The Rust adaptor projects package their exact source through a BLAKE3-fixed `.#source` selector while executing against the shipped Mantle libraries.

Project checks cover successful behavior and explicit invalid-input rejection. Build success proves only the selected local build and check; it does not prove compiler correctness, general cross-language equivalence, upstream trust, release signatures, or cross-platform reproducibility.
