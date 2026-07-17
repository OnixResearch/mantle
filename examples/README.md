# Mantle examples

This directory is a supported examples gallery. `examples/catalog.ncl` is the source of truth for support tiers, prerequisites, and validation rails.

## Gradual build path

Begin with the local seed-free stages, then continue into pinned network inputs and source-built toolchains. Each stage keeps the earlier concepts and introduces one new build concern.

| Stage | Example | New concept |
|---|---|---|
| Foundation | `examples/hello.ncl` | one derivation and one flat output |
| Structure | `examples/multi-step.ncl` | multi-command builder script |
| Configuration | `examples/build-environment.ncl` | declarative builder environment and directory output |
| Executable | `examples/cowsay.ncl` | installing and executing an output script |
| Artifact | `examples/static-site.ncl` | multi-file directory artifact |
| Parallel roots | `examples/multiple-roots.ncl` | independent top-level derivations |
| Named outputs | `examples/local-output-layout.ncl` | one derivation with runtime, development, and documentation outputs |
| Dependency | `examples/dependency-chain.ncl` | producer-to-consumer build ordering and mounted inputs |
| Selection | `examples/selected-output.ncl` | mounting one named output with `mantle.select` |
| Graph | `examples/diamond-dependency.ncl` | shared dependency reuse across converging branches |
| Project package | `examples/projects/generated-site/mantle-project.ncl` | default package and separately selectable check |
| Project pipeline | `examples/projects/codegen-pipeline/mantle-project.ncl` | model, code generation, application, and check selectors |
| External source | `examples/projects/fetched-and-patched/mantle-project.ncl` | fixed-output fetch, local patching, and rejected invalid context |
| Output hygiene | `examples/projects/multi-output-sdk/mantle-project.ncl` | runtime, development, documentation, and debug outputs with selected consumers |
| Language boundary | `examples/projects/schema-codegen/mantle-project.ncl` | one schema generating checked C and Rust applications |
| Release artifact | `examples/projects/reproducible-release/mantle-project.ncl` | normalized archives, BLAKE3 identity, and tamper detection |
| Offline handoff | `examples/projects/offline-source-bundle/mantle-project.ncl` | source-bundle planning, pinned import, readiness, and tamper rejection |
| Reviewed generation | `examples/projects/reviewed-file-generation/mantle-project.ncl` | non-mutating plans, drift-checked apply, and managed file identity |
| Developer loop | `examples/projects/developer-shell-run/mantle-project.ncl` | package execution and named shell activation |
| Existing Cargo migration | `examples/projects/cargo-import-offline/workflow.ncl` | review-first Cargo plan/apply with explicit input placeholders |
| Foreign handoff | `examples/projects/foreign-import-handoff/workflow.ncl` | frontend-free validation and receipt-bound planning |
| Portable evidence | `examples/projects/portable-receipt-handoff/mantle-project.ncl` | receipt archive verification/import plus semantic graph queries |
| OCI projection | `examples/projects/kernel-bundle-oci-local/workflow.ncl` | admitted local objects, atomic OCI export, fresh-state import, and descriptor-tamper rejection |
| Remote realization | `examples/projects/remote-build-loopback/mantle-project.ncl` | one-use ticket, framed stdio dispatch, signed admission, and redacted status |
| WebAssembly component | `examples/projects/wasm-component-hello/workflow.ncl` | typed export plus pinned materialization and drift rails |

## Beginner

Start here. These examples are local, fast, and do not need generated seed material unless the capability column says so.

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/hello.ncl` | `mantle build examples/hello.ncl --no-substitute` | flat file containing `Hello, mantle!` | local + fast |
| `examples/multi-step.ncl` | `mantle build examples/multi-step.ncl --no-substitute` | flat file containing `name: multi-step` | local + fast |
| `examples/build-environment.ncl` | `mantle build examples/build-environment.ncl --no-substitute` | directory containing a configured message and build-mode metadata | local + fast |
| `examples/cowsay.ncl` | `mantle build examples/cowsay.ncl --no-substitute` | runnable `bin/cowsay` script with default and custom messages | local + fast |
| `examples/static-site.ncl` | `mantle build examples/static-site.ncl --no-substitute` | directory containing `index.html` and `assets/site.css` | local + fast |
| `examples/mk-hello.ncl` | `mantle build examples/mk-hello.ncl -I examples -I builders --no-substitute` | store path containing `bin/hello` | generated seed |
| `examples/transcripts/hello-eval.md` | `mantle transcript run examples/transcripts/hello-eval.md --output /tmp/mantle-hello-transcript.json` | versioned transcript evidence containing the evaluated `hello` derivation | local + fast |

## Diagnostics

| File | What it shows | Expected result |
|---|---|---|
| `examples/fail.ncl` | A builder that fails intentionally. | negative build diagnostic |

## Fetcher cookbook

Real-network examples stay useful for operators, but deterministic validation uses generated offline fixtures in `tests/examples_build.rs` for each fetcher helper family. For disconnected rehearsal, pair a fetcher example with the offline build runbook: `mantle source bundle export --build-root examples/fetch-crate-crc64.ncl --import-path lib --to source-bundle.json`, `mantle source bundle import --from source-bundle.json --pin`, `mantle source bundle verify --from source-bundle.json --imported`, `mantle source bundle preflight --build-root examples/fetch-crate-crc64.ncl --import-path lib`, then `mantle build --offline-source-preflight --no-substitute examples/fetch-crate-crc64.ncl`. Inspect `ready_class`, `source_state_blake3`, `next_actions[]`, `network_policy_reports[]`, `cargo_build_evidence[]`, and `cargo_build_evidence_diagnostics[]`; the source bundle evidence proves declared source/input availability and identity only, and source-bundle route execution is future work.

| File | Command | Expected output shape | Capability | Offline validation rail |
|---|---|---|---|---|
| `examples/fetch-file.ncl` | `mantle build examples/fetch-file.ncl` | fixed-output file | real network | `offline-fetchurl-fixture` + `fixed-output-negative` |
| `examples/fetch-tarball.ncl` | `mantle build examples/fetch-tarball.ncl` | unpacked source tree | real network | `offline-fetch-tarball-fixture` + `fixed-output-negative` |
| `examples/fetch-git.ncl` | `mantle build examples/fetch-git.ncl` | checkout tree without `.git/` | real network | `offline-fetchgit-fixture` + `fixed-output-negative` |
| `examples/fetch-crate-crc64.ncl` | `mantle build examples/fetch-crate-crc64.ncl` | crate source tree containing `Cargo.toml` | real network | `offline-fetch-tarball-fixture` + `fixed-output-negative` |

## Package composition

After basic derivations and fetchers, move to output layouts and package relationships.

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/multiple-roots.ncl` | `mantle build examples/multiple-roots.ncl --no-substitute` | two independent flat output paths | local + fast |
| `examples/local-output-layout.ncl` | `mantle build examples/local-output-layout.ncl --no-substitute` | named outputs with `bin/show-layout`, `include/local_output_layout.h`, and `share/doc/local-output-layout/README` | local + fast |
| `examples/dependency-chain.ncl` | `mantle build examples/dependency-chain.ncl --no-substitute` | `result.txt` combining producer and consumer messages | local + fast |
| `examples/selected-output.ncl` | `mantle build examples/selected-output.ncl --no-substitute` | selected development header copied into the consumer artifact | local + fast |
| `examples/diamond-dependency.ncl` | `mantle build examples/diamond-dependency.ncl --no-substitute` | `graph.txt` combining both branches built from one shared input | local + fast |
| `examples/build-from-source.ncl` | `mantle build examples/build-from-source.ncl -I examples --no-substitute` | installed library and binary | generated seed |
| `examples/multi-output.ncl` | `mantle build examples/multi-output.ncl -I examples --no-substitute` | named outputs: `out`, `dev`, and `man` | generated seed |
| `examples/override.ncl` | `mantle eval examples/override.ncl -I examples -I builders` | overridden derivation metadata | generated seed |
| `examples/package-set.ncl` | `mantle eval examples/package-set.ncl -I examples -I builders` | related package records | generated seed |

## Project workflow

Project examples show selector syntax after package composition. See `examples/projects/README.md` for the complete project index and `examples/project/README.md` for the compatibility project command table.

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/projects/generated-site/mantle-project.ncl` | `cd examples/projects/generated-site && mantle build` | generated site with `index.html`, CSS, and manifest | local + fast |
| `examples/projects/codegen-pipeline/mantle-project.ncl` | `cd examples/projects/codegen-pipeline && mantle build .#checks.app` | model-to-generated-source application graph and check result | local + fast |
| `examples/projects/c-library-cli/mantle-project.ncl` | `cd examples/projects/c-library-cli && mantle build .#checks.test-greet` | static C library, CLI, public header, unit test, and project check | heavy + first-build network |
| `examples/projects/rust-workspace/mantle-project.ncl` | `cd examples/projects/rust-workspace && mantle build .#workspace-app` | offline Cargo-built library/CLI workspace | heavy + first-build network |
| `examples/projects/fetched-and-patched/mantle-project.ncl` | `cd examples/projects/fetched-and-patched && mantle build .#checks.patch` | verified upstream source with a BLAKE3-fixed local patch | real network |
| `examples/projects/multi-output-sdk/mantle-project.ncl` | `cd examples/projects/multi-output-sdk && mantle build .#checks.development` | selected runtime/dev consumers plus SDK `out`, `dev`, `doc`, and `debug` outputs | heavy + first-build network |
| `examples/projects/schema-codegen/mantle-project.ncl` | `cd examples/projects/schema-codegen && mantle build .#checks.integration` | generated C/Rust bindings, two CLIs, and cross-language behavior check | heavy + first-build network |
| `examples/projects/reproducible-release/mantle-project.ncl` | `cd examples/projects/reproducible-release && mantle build .#checks.reproducible` | byte-identical normalized archives and a BLAKE3 sidecar | local + fast |
| `examples/projects/signed-cache-roundtrip/mantle-project.ncl` | follow the project-local runbook | signed NAR publication, fresh-store substitution, and rejected untrusted/corrupt entries | heavy + loopback HTTP |
| `examples/projects/locked-dependency-lifecycle/mantle-project.ncl` | `cd examples/projects/locked-dependency-lifecycle && mantle check` | checked lock/generated-input state with offline stale/refresh lifecycle | local + fast |
| `examples/projects/offline-source-bundle/mantle-project.ncl` | follow the project-local runbook | source-bundle plan/export/verify/import/preflight with tamper rejection | local + fast |
| `examples/projects/reviewed-file-generation/mantle-project.ncl` | `cd examples/projects/reviewed-file-generation && mantle filegen plan --plan-out /tmp/filegen-plan.json` | non-mutating plan and drift-checked generated config apply | local + fast |
| `examples/projects/developer-shell-run/mantle-project.ncl` | `cd examples/projects/developer-shell-run && mantle run .#tool -- Mantle` | runnable package plus `dev` and `minimal` shell profiles | local + fast |
| `examples/projects/cargo-import-offline/workflow.ncl` | follow the project-local scratch-copy runbook | deterministic Cargo import plan/apply and explicit unresolved input roles | local + fast |
| `examples/projects/foreign-import-handoff/workflow.ncl` | `cd examples/projects/foreign-import-handoff && mantle --json foreign-import plan ...` | accepted receipt-bound adapter plan without Guix or Nix commands | local + fast |
| `examples/projects/portable-receipt-handoff/mantle-project.ncl` | follow the project-local archive/receipt/graph runbook | diagnostic receipt verify/import reports and bounded graph explanations | local build + fast CLI |
| `examples/projects/kernel-bundle-oci-local/workflow.ncl` | follow the project-local import/export/import runbook | sealed projection, OCI descriptor identities, admitted fresh-state object refs, and tamper rejection | local + fast |
| `examples/projects/remote-build-loopback/mantle-project.ncl` | follow the project-local one-use ticket runbook | framed stdio remote result, signed admission, redacted status, and exhausted-ticket rejection | heavy + local bwrap |
| `examples/projects/wasm-component-hello/workflow.ncl` | `cd examples/projects/wasm-component-hello && mantle eval workflow.ncl` | typed generated-input export; production rail publishes rehashable component evidence | typed export fast; production heavy |
| `examples/projects/cross-compiled-host-tool/mantle-project.ncl` | `cd examples/projects/cross-compiled-host-tool && mantle build .#target` | host-generated header consumed by an `x86_64-linux-musl` executable | heavy + first-build network |
| `examples/projects/store-gc-lifecycle/mantle-project.ncl` | follow the project-local runbook | persistent roots, dry-run candidates, collected unreachable output, and mutation-lock rejection | heavy + local state |
| `examples/projects/delta-substitution/mantle-project.ncl` | `cargo run -p mantle --example delta_substitution` | partial chunk transfer, full fallback, and fail-closed missing-chunk rejection | Rust adaptor |
| `examples/projects/release-witness-handoff/mantle-project.ncl` | `cargo run -p mantle --example release_witness_handoff` | signed release/witness handoff with quorum and revocation outcomes | Rust adaptor |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build` | default package store path with `bin/hello` | generated seed |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build .#hello` | named package store path with `bin/hello` | generated seed |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build .#goodbye` | named package store path with `bin/goodbye` | generated seed |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build .#checks.test-hello` | check output with `result` text `ok` | generated seed |
| `examples/rust_compatibility_rail.rs` + `examples/rust_compatibility_surface_matrix.ncl` | `cargo test -p mantle --test rust_compatibility_rail` | generated representative Rust compatibility rail surface matrix; sandboxed offline Cargo smoke plus rust-plan bounded success/blocker receipt | fast + bwrap |
| `examples/hardware_simulation_plan.rs` | `cargo run -p mantle --example hardware_simulation_plan -- request.json plan.json` | bounded generic `mantle-plan-v1` JSON with independent compile, link, and smoke units | fast planning; real Verilator rail is heavy + bwrap |

The hardware plan generator consumes a JSON `HardwarePlanRequest`; it does not interpret HDL in Mantle core. The exact request construction, tool cohort, real execution command, evidence shape, seed boundary, and non-claims are documented in [`docs/hardware-simulation.md`](../docs/hardware-simulation.md).

The Cargo import workflow stops at generated, reviewable Mantle files with explicit failing source/toolchain placeholders; the existing Rust workspace project owns the separately supported offline build lane. The Wasm workflow's checked-in Nickel file proves typed authoring/export only, while `tests/wasm_component_cli.rs` owns production execution against the pinned cohort.

The representative Rust compatibility rail is lane-scoped evidence, not proof of
full Cargo compatibility, compiler correctness, release reproducibility, or
bootstrap correctness. `examples/rust_compatibility_surface_matrix.ncl` is the
source-controlled surface matrix: it names supported offline Cargo surfaces,
native `cargo-free-bounded-topology` path-workspace support, blocked surfaces
such as vendored git and native-link metadata, stable blocker classes, and
non-claims. The offline rail reports `cargo-inside-mantle-sandbox`; the native
rail reports either `cargo-free-bounded-topology` or a deterministic
`blocked-unsupported-surface` receipt.

## Trust/provenance

These commands inspect local build evidence. They are not release or witness proofs, and placeholder output must not be treated as proof evidence.

| Example or recipe | Command | Expected evidence shape | Non-claim |
|---|---|---|---|
| JSON build report for `examples/hello.ncl` | `mantle --json build examples/hello.ncl --store /tmp/mantle-examples-store --state-dir /tmp/mantle-examples-state --no-substitute` | build-report JSON with `outputs[].artifact_attestation.path` | proves only local build/report shape |
| `examples/crunch.ncl` | `mantle eval examples/crunch.ncl -I examples` | self-build derivation skeleton shape | does not prove release, witness, or fixed-point self-hosting success |
| `examples/projects/signed-cache-roundtrip/mantle-project.ncl` | follow its README producer/cache/consumer workflow | narinfo signature and content-hash acceptance | does not prove producer correctness or upstream trust |
| `examples/projects/offline-source-bundle/mantle-project.ncl` | follow its README producer/consumer source-state workflow | BLAKE3-bound source records and offline readiness | does not prove build success, source trust, or output correctness |
| `examples/projects/reviewed-file-generation/mantle-project.ncl` | plan then apply in a scratch project copy | reviewed content identity and managed-file ownership | does not prove deployability or downstream semantic correctness |
| `examples/projects/developer-shell-run/mantle-project.ncl` | run the package and activate both shell profiles | explicit package arguments and sidecar environment activation | does not prove hermeticity or release reproducibility |
| `examples/projects/artifact-provenance-walkthrough/mantle-project.ncl` | follow its README build/show/closure/verify/diff workflow | canonical artifact and closure sidecars linked to selected store outputs | does not prove builder, source, dependency, or compiler correctness |
| `examples/projects/hermetic-plan-rebuild/mantle-project.ncl` | compare strict plans and builds across fresh state | stable action/output identities and no hermeticity audit events | does not prove compiler correctness, sandbox completeness, or cross-platform reproducibility |
| `examples/projects/shared-action-result-roundtrip/mantle-project.ncl` | follow its README producer/static-cache/consumer workflow | signed action record plus independently trusted PathInfo/NAR evidence | does not prove executor correctness or turn action metadata into artifact trust |
| `examples/projects/store-gc-lifecycle/mantle-project.ncl` | follow its README root/GC workflow | state-scoped root and collection report | does not prove distributed retention |
| `examples/projects/delta-substitution/mantle-project.ncl` | `cargo run -p mantle --example delta_substitution` | in-memory delta/fallback transfer report | does not prove HTTP cache interoperability |
| `examples/projects/release-witness-handoff/mantle-project.ncl` | `cargo run -p mantle --example release_witness_handoff` | synthetic signed attestation directory and policy result | does not prove release evidence or an independent rebuild |
| `examples/projects/foreign-import-handoff/workflow.ncl` | validate and plan the checked Guix-like and Nix-like fixtures | raw-graph digest plus receipt-bound adapter plan | does not prove output trust, frontend correctness, or rebuild success |
| `examples/projects/portable-receipt-handoff/mantle-project.ncl` | verify/import the receipt against a store archive, then query both graph fixtures | diagnostic receipt matches plus complete/incomplete graph results | does not prove execution, compiler, payload-transfer, or release correctness |
| `examples/projects/kernel-bundle-oci-local/workflow.ncl` | import the fixture objects, export the sealed projection, then import into fresh state | BLAKE3 object refs plus separate OCI SHA-256 descriptors and a round-trip response | does not publish a registry artifact or prove kernel compatibility, bootability, deployability, signatures, or release eligibility |
| `examples/projects/remote-build-loopback/mantle-project.ncl` | dispatch with a one-use ticket through the default local stdio worker | remote route, signed output admission, artifact reference, and redacted status | does not prove production P2P, restart, SSH, or resumable-transfer deployment |
| `examples/projects/wasm-component-hello/workflow.ncl` | evaluate the typed export, then run the pinned production CLI rail | generated-input ownership and rehashable materialization bundle | does not prove behavior correctness, runtime authority, compiler correctness, or release eligibility |
| `examples/transcripts/hello-eval.md` | run through `mantle transcript run` | isolated versioned command transcript | does not prove the evaluated derivation was built |

## Advanced bootstrap

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/bootstrap-no-nix.ncl` | `mantle build examples/bootstrap-no-nix.ncl --no-substitute` | store path containing `bin/hello` | heavy + bwrap |
| `examples/build-crate-crc64.ncl` | `mantle build examples/build-crate-crc64.ncl --no-substitute` | store path containing `bin/crc64` | heavy + real network + bwrap |
| `examples/hello-static.ncl` | `mantle build examples/hello-static.ncl -I examples --no-substitute` | static `hello` binary | generated seed |
| `examples/hello-world.ncl` | `mantle build examples/hello-world.ncl -I examples --no-substitute` | C hello binary | generated seed |
| `examples/seed.ncl` | generated by `mantle bootstrap -o examples/seed.ncl` | host-specific seed paths | generated support file |

## Benchmarks

| File | What it shows | Validation |
|---|---|---|
| `examples/benchmark_eval_smoke.rs` | Cheap evaluation benchmark bundle. | `tests/benchmark_harness.rs` |
| `examples/benchmark_suite.rs` | Full checked-in benchmark workload matrix. | `tests/benchmark_harness.rs` |
| `examples/benchmark_compare.rs` | Compares two benchmark bundles. | `tests/benchmark_harness.rs` |
| `examples/benchmark_eval_backends.rs` | Evaluation backend benchmark. | `tests/benchmark_harness.rs` |
| `examples/benchmark_lazy_eval.rs` | Lazy selected-root evaluation benchmark. | `tests/benchmark_harness.rs` |
| `examples/benchmark_scheduler_priority.rs` | Scheduler-priority benchmark. | `tests/benchmark_harness.rs` |

## Common commands

```bash
# Evaluate only
mantle eval examples/fetch-crate-crc64.ncl

# Build into writable temp roots
mkdir -p /tmp/mantle-examples-store /tmp/mantle-examples-state

# Fetch a real crate source tarball
mantle build examples/fetch-crate-crc64.ncl \
  --store /tmp/mantle-examples-store \
  --state-dir /tmp/mantle-examples-state

# Build a real Rust crate from crates.io; heavyweight, run explicitly
mantle build examples/build-crate-crc64.ncl \
  --store /tmp/mantle-examples-store \
  --state-dir /tmp/mantle-examples-state \
  --no-substitute
```
