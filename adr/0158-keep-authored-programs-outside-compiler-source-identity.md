# 0158. Keep authored source programs outside compiler source identity

Status: Accepted

## Context

Leviathan task 1694 failed closed before protected execution because the source
recipe's semantic-unit inventory differed from the independently replanned
Cargo-free inventory. Preparation had planned the original Mantle source, then
written two recipes, two unit-plan documents and the cold selector into that
same tree. Runtime correctly hashed the augmented compiler source instead.

The retained eight root-package units used path-source digest
`ab44854e554d9c855b93b7e43abbc158c14938a23fab7ddbe96b38190c9e818c`;
the augmented tree hashed to
`4b5f41d1178406f4f718666bd06182d62f6217dc106ed3e20d80e7baaf30cc39`.
Read-only reconstruction using the existing Rust framing recovered the first
digest when exactly the five post-planning additions were omitted. Replanning
after writing recipes cannot resolve this: their compiler metadata and semantic
keys would depend on the bytes of the recipes containing those same keys.

The current consumer is the source-built fixed-point workflow. The required
outcome is identical source-owned unit identities during preparation and fresh
runtime admission, not acceptance of a stale or approximate inventory.

## Decision Drivers

- Preserve complete, exact compiler-source identity and semantic-unit admission.
- Authenticate every recipe, plan, helper and selector as independent source.
- Reuse the existing program record, chunk transport and `ManifestSource` reader.
- Reject old expanded cohorts rather than adding compatibility or hash exceptions.
- Fail during preparation if authoring changes the original compiler source.

## Decision

Preparation never modifies or recaptures the original Mantle source-role record.
After authoring it checks that the compiler tree retains the original plan's
exact size and content identity. No source hash exclusions or semantic-key
normalization are introduced.

The existing `source-built-mantle-programs` record carries its helper sources,
both complete Mantle recipes and pre-lowering unit plans, and
`bootstrap/source-built-cold-route.json`. The record is authenticated in the
native source-authority manifest and output profile, with existing canonical
chunking for large members. No compiler source tree receives these additions.

The selector uses `mantle-source-built-cold-route-v2`. Runtime resolves it from
that exact program record, and its two Mantle recipe selectors are typed
`ManifestSource` references to their exact owning-stage members in the same
record. Both recipes use the existing reference-bound preparation path. No
fallback reads the old selector or recipes from the compiler source tree.
The existing independent Cargo-free topology, compiler metadata, source-role,
producer, reachability, protected-event and reconciliation checks remain intact.

This supersedes ADR 0157's placement of unit-plan members in the Mantle
source-role record, not its requirement to retain complete pre-lowering facts.

## Consequences

- Mantle's source-program authoring and proof-inspection adapters own the
  boundary. The durable capability is repeatable source planning without a
  circular dependency on its own generated instruction data.
- Existing expanded cohorts require regeneration from original sources under a
  newly published and qualified source revision. Changed bytes require fresh
  complete current-data independent review before protected execution.
- Regression coverage exercises compiler-unit identity stability, rejection of
  compiler-source mutation, and typed selector authority without an old-source
  fallback. Current-data inspectors additionally compare the immutable original
  record/tree and every independently authenticated program member.
- Preparation, static inspection and synthetic contract tests are not protected
  execution, stage equality, compiler correctness, a verified v2 proof, native-row
  qualification or release authority.
