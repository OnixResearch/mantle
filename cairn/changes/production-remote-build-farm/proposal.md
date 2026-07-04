## Why

Mantle has the right distributed-build primitives, but the operator path is not yet a production build farm. The current remote-build work proves concrete build requests, framed stdio/SSH transport, source-bundle input sync, output admission, route planning, and coordinator data models. The remaining gap is the farm boundary that Hydra and Hercules CI already provide: scheduler-integrated remote placement, persistent worker coordination, large-object transfer, cryptographic output trust, cache publication, and operator status that survives restarts.

Hydra gives Nix users a mature central queue runner, jobsets, machine status, logs, and SSH remote builders. Hercules CI gives users a hosted/agent model where agents dial out, share binary caches, run evaluations/builds concurrently, and expose job status. Mantle should compete by preserving its stronger proof boundary: remote builders receive concrete Mantle build inputs, not frontend eval requests; input/output bytes are content-addressed; output trust is separate from resource access; and every accepted result goes through local PathInfo, castore, and attestation admission.

## What Changes

- Move remote realization from the sidecar CLI path into the lazy scheduler / build service dispatch path so remote work participates in goal dedupe, `-j` bounds, dependency interleaving, fallback, terminal-state propagation, and ordinary build reporting.
- Require real cryptographic output verification for remote PathInfo and attestation admission instead of treating builder key names or resource tickets as trust.
- Replace bounded inline payload exchange as the production path with resumable, streaming CAS/NAR/delta transfer with digest verification, quotas, and backpressure.
- Promote the coordinator/worker model into a persistent runtime: worker-initiated registration, capability matching, queue state, leases, bounded logs, restart adoption, and redelivery of finished results.
- Publish accepted remote results through configured artifact/cache publishers after local verification, while keeping publication failure separate from build success.
- Add a production-grade operator evidence rail that shows where Mantle is equivalent to Hydra/Hercules operationally and where Mantle deliberately has stronger trust boundaries.

## Impact

- **Files**: `src/remote_build.rs`, `src/main.rs`, `src/build_plan.rs`, `src/realization_routing.rs`, `crates/crunch-build/src/{distributed,orchestrate,dispatch_build_service}.rs`, `crates/crunch-store/src/{handle,pull,push,archive}.rs`, remote-builder docs, operator workflows, and this Cairn remote-builds spec delta.
- **Testing**: scheduler integration positives/negatives, cryptographic trust positives/negatives, streaming transfer positives/negatives, persistent coordinator restart tests, cache-publication tests, and an operator e2e rail with bounded local multi-process fixtures.

## Out of Scope

- Hosted SaaS control plane.
- Web UI parity with Hydra.
- Arbitrary build-system action remoting below the Mantle derivation/action boundary.
- Treating remote builder authorization, coordinator assignment, or cache publication as output correctness proof.
