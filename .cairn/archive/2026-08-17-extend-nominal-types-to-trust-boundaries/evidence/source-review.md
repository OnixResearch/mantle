# Nominal trust-boundary source review

## Question

Which Mantle primitive fields have enough role, grammar, unit, or authority to justify nominal admission?

## Inspected evidence

- `crates/crunch-build/src/dynamic_plan.rs:186-233` uses role-specific BLAKE3 types.
- `crates/crunch-build/src/dynamic_plan/wire.rs:31-49` separates structural wire values from one pure admitted model.
- `docs/nominal-dynamic-plan-types.md` records the accepted compatibility and claim boundary.
- `crates/crunch-bootstrap-core/src/lineage.rs:35-50` defines an infallible `Blake3Hex::new` and checks format later.
- `crates/crunch-bootstrap-core/src/lineage.rs:122-237` keeps source, generated-artifact, tool, patch, assumption, and graph-node IDs as strings.
- `crates/crunch-bootstrap-core/src/stagex.rs:25-94` keeps stage, artifact, authorization, path, limit, status, and digest roles as primitives.
- `src/remote_build.rs` repeats raw request, session, endpoint, ticket, output, and transfer identities across protocol records.
- `crates/crunch-release-core/src/content_bound_requirements.rs` validates repository IDs, requirement IDs, specification paths, release IDs, revisions, and BLAKE3 text after deserialization.
- `src/frontend_artifact_spec.rs:27-82` copies validated spec, validator, hash, artifact, target, and build-root text into attestations.
- `crates/crunch-build/src/fetcher.rs:385-443` and `:1227-1303` suppress ambiguous-parameter findings for URL, revision, archive, and output roles.
- `crates/crunch-build/src/ca_plan.rs:41-82` and `crates/crunch-build/src/rewrite.rs:30-39` use interchangeable names and provisional or final paths.

## Decision

Create one cross-cutting Cairn change under `build-correctness`. Reuse the dynamic-plan wire-to-admitted pattern in bounded ownership slices. Keep credential bearer and verifier values in `harden-remote-credential-boundary`.

Use crate-local types with fallible constructors. Use typed aggregates for relational invariants. Use enums for closed statuses or linked algorithm/value records. Do not create blanket wrappers for free-form text.

## Owner

`extend-nominal-types-to-trust-boundaries`

## Next action

Capture compatibility baselines, then implement StageX and lineage admission first. Continue through remote protocol, evidence, frontend, and helper boundaries only after each prior slice has focused positive and negative evidence.

## Non-claims

This review does not prove implementation completion, wire compatibility, external authority, artifact correctness, build success, or release eligibility.
