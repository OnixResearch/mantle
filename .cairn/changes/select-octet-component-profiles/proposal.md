# Proposal: Select Octet component profiles by request identity

## Why

Molten's real actor-world producer fixture needs the preserves-payload type import, bulk memory and LLVM's padded zero indirect-call table immediate. The import-free portable profile correctly denies it. Octet published the narrowly scoped profile in `0e0e5dc8461b8b32c286de362c290b3f3ea2deee`; Mantle currently admits only its portable default and incorrectly repeats that default in downstream evidence.

## What Changes

- Pin that immutable Octet source and regenerated registry.
- Build a toolchain cohort declaring the portable and Molten artifact profiles, each identified through actual independent Octet collection and verification.
- Resolve each request's existing expected artifact-profile identity without default fallback; carry that exact selection through invocation, admission, evidence and materialization.
- Reject missing, duplicated, misnamed, stale or unknown profile bindings before component execution.

## Impact

Immediate consumer: Molten's supported actor-world build, not shared-runtime activation. Maintenance owner: Mantle component-pipeline/toolchain maintainers; Octet owns artifact policy, and Molten owns product runtime admission. Repeatability comes from pinned toolchain realization, actual producer/consumer observations and focused boundary regressions.

## Non-goals

No portable-baseline weakening, broad Wasm 2 admission, product authority grant, actor/extension installation, runtime cutover, release eligibility or compiler-correctness claim.
