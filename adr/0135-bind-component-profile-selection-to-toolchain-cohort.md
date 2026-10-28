# ADR 0135: Bind component artifact profile selection to the exact toolchain cohort

## Status

Proposed

## Context

Molten's actor world uses a declared Preserves carrier type import, bulk memory and LLVM's padded zero indirect-call table immediates. Octet's import-free portable baseline correctly denies it. A separate narrowly scoped Octet profile is available, but Mantle currently treats its default as the only selectable artifact policy and repeats it in validation evidence.

## Decision Drivers

- Keep artifact policy owned by Octet and runtime admission owned by Molten.
- Preserve the restrictive portable baseline.
- Bind request, invocation, receipt and bundle to one exact profile identity.
- Reject stale or unknown selections before execution, without implicit migration.

## Decision

Pin Octet source and registry bytes and include exactly the portable and Molten profile identities in the generated toolchain manifest. Generate those identities through independent real Octet collection and artifact verification. Validate both names and exact identities, not merely the manifest's self-hash.

Use the request's existing expected artifact-profile identity to borrow the corresponding cohort entry. No profile-name guessing, default fallback or old-manifest compatibility shim is admitted. Carry the selection through invocation, receipt validation, stage and published evidence, portable admission and materialization. Keep the runtime-profile identity independent.

## Consequences

Both profiles participate in the toolchain cohort hash. A cohort update requires regenerated requests and fresh producer evidence; old bundles are not relabeled. Consumers may retain the portable default explicitly or declare the separate Molten identity. Maintenance belongs to Mantle's component-pipeline/toolchain owners; Octet retains static admission policy and Molten retains runtime/authority decisions.

This enables repeatable supported-world materialization. It does not establish execution correctness, actor/extension adoption, release eligibility or product authority.
