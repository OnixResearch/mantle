# Proposal: Qualify distributed compilation

## Why

A local stdio loopback, two endpoint labels, passing scheduler tests or a signed
output do not show that one project was compiled across independent hosts.
The current remote loopback documentation explicitly excludes independent
machines. The new pool and compiler frontends need one repeatable operational
acceptance path with retained failures, actual compiler observations and bounded
claims, not an inferred speedup or a hand-written success summary.

## What Changes

- Add a repository-owned qualification runner driven by a typed, exported
  Nickel manifest naming exact workloads, tools, policies and operator-supplied
  independent worker endpoints. r[distributed_compilation.qualification_manifest]
- Run real Rust and C/C++ local/one-/two-worker builds, including a
  mixed-language Rust consumer of a declared Mantle-built native library,
  and verify final program behavior. r[distributed_compilation.multihost_execution]
- Exercise failure, trust, privacy, capacity, cancellation and recovery controls
  through the production adapters with current attempt/lease observations.
  r[distributed_compilation.qualification_failures]
- Repeat from fresh stores, measure execution/transfer costs and publish a
  bounded support/evidence matrix without promoting unavailable cells or
  measurements into broader claims. r[distributed_compilation.repeatable_evidence]

## Impact

- **Immediate consumers:** `schedule-native-goals-across-workers`,
  `distribute-rust-compilation-units` and `distribute-cc-compilation-units`,
  plus operators deciding whether to enable the resulting opt-in pool path.
- **Immediate outcome:** a reproducible independent-host acceptance command
  with exact inputs, working final programs and inspectable negative controls.
- **Durable capability:** a maintained distributed-compilation qualification
  rail that can be rerun after toolchain, protocol, frontend or store changes.
- **Maintenance owner:** Mantle build/evidence owners, with frontend owners
  responsible for their workload/support rows and remote owners for adapter
  and authority controls.
- **Repeatability evidence:** complete run manifests, raw bounded transcripts,
  BLAKE3 identity/linkage, exact compiler attempts, accepted outputs, observed
  teardown, failed/unavailable cells and a fresh-state repeat.
- **Compatibility:** opt-in heavy proof workflow; ordinary tests and default
  local builds do not require remote machines or silently launch this campaign.

## Dependencies

`metadata.json` requires both compiler frontend changes. They transitively
require pool dispatch and its existing native-remote prerequisites. Structural
proposal/design/tasks gates do not satisfy those dependencies or runtime
acceptance. See
[proposed ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).

## Non-Goals

Choosing a public service, provisioning hosts or secrets, authorizing uploads
from a hostname, REAPI/distcc compatibility, all-target/full-workspace support,
compiler correctness, arbitrary worker honesty, full-source bootstrap,
release eligibility or a general performance guarantee.

## Success Criteria

Two operator-supplied independent worker hosts execute overlapping compiler
work for one selected graph; final Rust, C/C++ and mixed-language programs run
with expected results. Fresh cold and controlled warm runs have complete
identity/admission/count evidence. Wrong keys/bytes/fences, compiler failures,
resource exhaustion, disconnect/cancel and private-input attempts cannot
publish a false success. Missing independent hosts or credentials produces an
explicit nonpassing unavailable qualification, not loopback acceptance.
