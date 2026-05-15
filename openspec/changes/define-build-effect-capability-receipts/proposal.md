## Why

Unison's ability model makes effects explicit before a program can run. Mantle's deterministic proof work has the same need: release, witness, and build receipts should not merely say a sandbox existed; they should record which effects were declared and which effects were observed. Today proof classes can reject direct-host or unsupported sandbox evidence, but there is no single effect vocabulary that lets operators see whether a recipe used network, clock, host tools, environment, secrets, or remote execution.

## What Changes

- **Build effect vocabulary**: Define a closed first-phase set of Mantle build/proof effects.
- **Receipt binding**: Require deterministic proof and release verification receipts to carry declared and observed effects.
- **Fail-closed verification**: Reject deterministic proof claims when observed effects exceed declared policy or when a receipt omits required effect evidence.

## Capabilities

### New Capabilities
- `build-effect-capability-receipts`: capability-aware deterministic proof receipts.

### Modified Capabilities
- `release-verification-tech`: deterministic proof reports include effect declarations and observations.
- `build-pipeline`: build execution exposes auditable effect observations.
- `provenance`: attestations can distinguish declared effect claims from observed effect facts.

## Impact

- **Files**: release verification core, sandbox/proof receipt schemas, build audit events, tests, docs.
- **APIs**: additive receipt fields and validation policy; no default distributed backend.
- **Testing**: positive receipt verification plus negative tests for undeclared network/clock/env/host-tool effects and missing observed-effect evidence.
