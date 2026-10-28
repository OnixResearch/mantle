# Design: Select Octet component profiles by request identity

## Context

The existing request already names an exact Octet artifact-profile identity. The toolchain exposed one default and preflight required equality with it. Selecting a different profile must change the toolchain cohort and cannot reinterpret an Octet finding.

## Decisions

1. Retain the existing portable default fields for default configuration; add a required, closed two-entry `profiles` cohort. It includes that default and `molten-component-baseline`. There is no missing-field compatibility fallback.
2. Pin Octet source, registry bytes, both profile identities and wasm-tools cohort. Verify exact names, cardinality, distinct identities and default association before the pipeline runs. Manifest self-hashing alone is not profile admission.
3. Select by the request's already-declared identity, borrowing the admitted manifest entry. Never allocate a synthetic default, select by application name, silently broaden policy or fall back on unknown identity.
4. Use the selected identity for real `cargo-octet --profile`, receipt validation, stage evidence, published Octet evidence, portable admission and final materialization. Runtime profile identity stays independent.
5. Build each selectable profile identity from a tiny real component with `evidence collect` and `artifact verify`. Toolchain generation requires registry/cohort agreement before recording it.

## Verification

Boundary regressions cover exact selection and rejection of absent, duplicate, misnamed and misbound cohorts. Actual producer observation must retain the supported-world portable-profile denial, succeed with the Molten profile, remeasure the materialization, verify Structural+Bytes and copy the genuine source tree. A default demo build must remain supported; unknown profile must fail before publication.

## Ownership and Rollback

Octet owns the proposal/import policy. Mantle owns profile selection and evidence binding, not runtime semantics. Roll back source, lock, profile constants and generated toolchain as one cohort; do not relabel old bundles. Molten's runtime-adoption change remains separate.
