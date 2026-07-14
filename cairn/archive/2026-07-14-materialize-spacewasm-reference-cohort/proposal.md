## Why

[NASA/JPL SpaceWasm](https://github.com/nasa/spacewasm) offers a no-std WebAssembly 1.0 decoder, validator, and interpreter with bounded instruction stepping, streaming decode, explicit host functions, and fixed-page allocation patterns that are relevant to ChaosControl, Molten, Octet, Trellis, Animus, and Kamacite. The upstream project currently has no release, reports workspace version `0.0.0`, and is changing quickly. Letting each consumer fetch a branch, build a different toolchain closure, or infer support from repository prose would make differential results and compatibility claims irreproducible.

Mantle should materialize one reviewed SpaceWasm reference cohort before any downstream repository treats it as executable evidence. The bundle must preserve exact source, build, feature, test, and non-claim identities without promoting NASA/JPL provenance or passing upstream tests into certification, runtime correctness, or release eligibility.

## What Changes

- Add a typed Nickel SpaceWasm reference-cohort manifest and deterministic runtime export.
- Pin an exact upstream commit, Cargo lockfile, Rust toolchain, target set, dependency closure, license/notice files, and reviewed feature-support snapshot.
- Build the no-std library and bounded host-side diagnostic runner without network access or floating dependency resolution.
- Materialize representative MVP modules, positive and negative streaming fixtures, upstream spectest/fuzz corpus descriptors, and deterministic runner inputs as separate artifacts.
- Emit a rehashable reference bundle with BLAKE3 identities for source, build closure, binaries, profiles, fixtures, reports, and parent edges.
- Record upstream test results and known gaps as bounded evidence, including absent releases, unsupported post-MVP features, missing `spacewasm-check`, unresolved performance work, and pending continuous fuzzing.
- Provide a stable consumer handoff for ChaosControl, Molten, Octet, Trellis, Animus, and Kamacite without transferring those repositories' runtime, policy, proof, or release semantics into Mantle.

## Impact

- **Surfaces**: Nickel materialization profiles, source acquisition, Rust build planning, fixture/corpus manifests, build reports, artifact attestations, reference bundles, and release-evidence non-claims.
- **Dependencies**: the motivating review observed upstream commit `30cd6e9b91f84a39278edcb5d66514b773011ccc`; implementation may select a later reviewed commit only through an explicit profile update and complete fixture replay.
- **Consumers**: downstream repositories remeasure exact bundle members and own their own admission decisions; a Mantle bundle is not runtime approval.
- **Claims**: successful materialization proves bounded source/build/artifact identity and recorded checks only. It does not prove SpaceWasm correctness, memory safety, WebAssembly conformance, flight qualification, sandbox effectiveness, or production readiness.
