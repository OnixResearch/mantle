# Validation: Mantle artifact-auth non-production canary evidence

Validated on 2026-07-20 against Mantle `55fb02decfa42325ed192ac1249e8a6432844d50` and artifact-auth `799459346d5416fbd7b9f55840a7371441b55afa`.

## Baseline

- Cairn validation passed before this evidence-only change.

## Evidence checks

- The self-contained public subset contains 12 regular files: the exact harness, real build report and output, operational receipt, capture/replay/revocation summaries, fresh-process denial log, typed manifest, bounded hash script, README, and inventory.
- `nix shell nixpkgs#nickel -c nickel typecheck .../manifest.ncl` passed.
- BLAKE3 inventory regeneration reproduced byte-for-byte; the negative symlink fixture failed with `symlink is forbidden`.
- JSON/log secret scanning found no private-key marker, serialized secret-key field, or long name-prefixed private-key value.
- The receipt retains `standalone_authority_admitted = false`; the fresh-process post-revocation log contains `CurrentnessNotCurrent`.
- The first inventory attempt exposed Mantle's repo-local `clang`/`mold` Cargo policy. The archived script now supplies GCC, Clang, and mold explicitly and passes from the repository without ambient linker assumptions.
- The exact build report retains historical local paths. The manifest and README classify them as observations, not live dependencies.

## Lifecycle checks

- Cairn validate plus proposal, design, and tasks gates passed.
- Sync dry-run passed with receipt `75828abfa76b2f080e4df86b553fb87c57f94c97d0ea3999a3f7c3a0a0fffc22`.
- Sync execution passed with receipt `0f662f25332bd056e62dd1244e6ae08de8fba40ee2a53cddb59e45d2f711e14f`.
- Accepted requirements `mantle.artifact_auth_operational_receipt.canary_archive` and `mantle.artifact_auth_operational_receipt.canary_authority` are present.

## Adversarial audit

The advisory audit usefully challenged self-containment and cross-consumer identity binding. Cross-consumer revisions are now explicitly review linkage rather than a joint signature or attestation. Its suggestion to clear `legacy_authoritative` was rejected because retaining legacy authority is the required fail-safe while standalone authority remains unadmitted. Deterministic checks remain authoritative.

## Bounded result

The exact non-production build, replay, and local revocation-denial evidence is durably reviewable without private signing material. This archive does not establish remote trust discovery, global revocation freshness, cache/build admission, registry publication, release eligibility, production rollout, or standalone authority. Legacy authority and rollback remain active.
