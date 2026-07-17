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
| `offline-source-bundle/` | `mantle source bundle export` | fresh-state verify, pinned import, preflight, and tamper rejection | local + fast |
| `reviewed-file-generation/` | `mantle filegen plan` | reviewed apply, state tracking, drift/conflict/escape rejection | local + fast |
| `developer-shell-run/` | `mantle run .#tool -- Mantle` | named shell activation and invalid-invocation rejection | local + fast |
| `cargo-import-offline/` | `mantle --json import cargo --plan` in a scratch workspace copy | reviewed apply plus ambiguous-package, missing-lock, stale-vendor, and conflict blockers | local + fast |
| `foreign-import-handoff/` | `mantle --json foreign-import validate` | Nix-like/Guix-like plans plus stale, untrusted, and unsupported-input rejection | local + fast |
| `portable-receipt-handoff/` | `mantle receipt bundle export` after `mantle build .#payload` | archive verification, idempotent import, graph explanation, and incomplete/conflict rejection | local + bwrap |
| `kernel-bundle-oci-local/` | `mantle artifact oci-export` after importing both fixture objects | atomic local layout, admitted fresh-state import, and descriptor-tamper rejection | local + fast |
| `kernel-bundle-oci-registry/` | `mantle artifact oci-push` followed by dual-digest `oci-pull` | authenticated image/metadata publication, exact layout recovery, admitted import, and drift/tamper/interruption rejection | networked registry; fast loopback validation |
| `artifact-provenance-walkthrough/` | `mantle --json build .#application` | show, closure, verify, diff, tamper, and missing-selector evidence | local + fast |
| `hermetic-plan-rebuild/` | `mantle --json build --plan --strict-hermetic .#payload` | fresh-state identity comparison and policy/source/prefix drift rejection | local + fast |
| `shared-action-result-roundtrip/` | `mantle --json build .#payload --publish-action-results` | static HTTP reuse, changed-action miss, unknown-signer rejection, and missing/corrupt-NAR fallback | heavy + loopback HTTP |
| `cross-compiled-host-tool/` | `mantle build .#target` | `mantle build .#role-mismatch` must fail | heavy + first-build network |
| `store-gc-lifecycle/` | `mantle build .#retained` | persistent root, dry-run, GC, and mutation-lock runbook | heavy + local state |
| `delta-substitution/` | `mantle build .#source` | `cargo run -p mantle --example delta_substitution` | Rust adaptor |
| `release-witness-handoff/` | `mantle build .#source` | `cargo run -p mantle --example release_witness_handoff` | Rust adaptor |
| `remote-build-loopback/` | `mantle build .#payload --builder gallery-builder --ticket <id:secret>` | production stdio streaming plus checked multi-chunk interruption/resume, repeated-content identity, tamper rejection, signed admission, and redacted status | heavy + local bwrap; interruption rail uses a debug build |
| `wasm-component-hello/` | `mantle eval workflow.ncl` | pinned production component materialization and identity/interface/runtime drift rails | typed export fast; production heavy |

The C, Rust, and SDK projects expose source or generated-source selectors that can be built without realizing every downstream package. Their compiled packages use Mantle-owned bootstrap/toolchain inputs rather than ambient host compilers or Cargo caches.

The fetched project pins both its unpacked upstream source and local patch. The reproducible release project builds independently named normalized archives, records the expected BLAKE3 digest, and includes a tamper-detection check.

The workflow tier adds signed cache exchange, lock refresh, offline source handoff, reviewed file generation, named development shells, Cargo import, foreign derivation admission, portable receipt/graph handoff, local OCI projection/import, artifact/closure evidence inspection, strict plan/rebuild comparison, authenticated shared action-result reuse, host/target role separation, persistent garbage collection, production local stdio streaming with fenced resume and fail-closed receiver tamper handling, typed component materialization, delta transfer policy, and signed witness quorum. The Rust adaptor projects package their exact source through a BLAKE3-fixed `.#source` selector while executing against the shipped Mantle libraries.

Source-bundle readiness proves only declared source availability, filegen plans prove only bounded generated-content identity, and shell activation is non-mutating convenience evidence. Artifact sidecars prove bounded identity/linkage facts, strict matching builds do not prove compiler correctness, and signed action records do not replace independent PathInfo/NAR trust. Project checks cover successful behavior and explicit invalid-input rejection. Build success proves only the selected local build and check; it does not prove general cross-language equivalence, upstream trust, release signatures, or cross-platform reproducibility.
