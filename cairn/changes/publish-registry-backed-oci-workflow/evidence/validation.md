# Implementation validation

Recorded: 2026-07-16

## Result

Mantle now publishes an admitted local OCI layout through a bounded Distribution v2 subset and restores it into fresh state through ordinary OCI admission. Push publishes the unchanged image plus a deterministic subject-bound metadata artifact, publishes the image tag last, re-verifies both immutable manifests, and emits a contracted receipt. Pull requires both immutable SHA-256 manifest expectations, verifies the complete descriptor/metadata closure, reconstructs exact layout bytes, and emits success only after an `admitted` import.

ADR 0028 records why the companion artifact and both immutable digest expectations are required. Registry routing and explicit credential mode remain separate from OCI SHA-256 and Mantle BLAKE3 content identities.

## Baseline

`evidence/baseline.md` records pueue task `56`. Existing local OCI core/shell/CLI rails passed before implementation. The machine-contract checker exposed three pre-existing remote JSON producers that lacked classification; this change classified those existing producers under the existing `remote.execution-reports` family rather than weakening the contract rail.

## Focused positive and negative evidence

Pueue task `130` ran the serialized focused chain over the staged implementation:

```text
cargo test -p mantle --lib 'oci_registry::tests::' -- --test-threads=1
cargo test -p mantle --bin mantle 'oci_registry_shell::tests::' -- --test-threads=1
cargo test -p mantle --test kernel_bundle_oci_registry_cli -- --test-threads=1
cargo test -p mantle --test kernel_bundle_oci_cli -- --test-threads=1
cargo test -p mantle --test examples_build -- --test-threads=1
cargo test -p mantle --test examples_inventory -- --test-threads=1
cargo test -p mantle --test examples_workflow_gallery -- --test-threads=1
cargo -Zscript scripts/check-machine-schema-contracts.rs
cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
```

The focused results included:

```text
registry core: 7 passed; 0 failed
registry shell: 2 passed; 0 failed
registry public CLI: 4 passed; 0 failed
local OCI public CLI: 4 passed; 0 failed
examples build: 23 passed; 0 failed; 1 ignored
gallery inventory: 17 passed; 0 failed
gallery workflows: 11 passed; 0 failed
machine schema contract check: PASS (18 contracted, 47 classified)
machine schema contract self-test: PASS
```

The public registry cases prove authenticated byte-identical push/pull into fresh admitted state, mutable image-tag rejection, metadata-blob tamper rejection before output/admission, missing credentials rejected with no receipt, interrupted image publication with no receipt, and a content-addressed retry that uploads zero duplicate blobs. Core/shell negatives also cover unsafe targets, escaped or ambiguous upload locations, wrong subjects, duplicate/wrong metadata roles and media types, malformed credentials, redirects/proxies, immutable receipt tampering, and schema accounting bounds.

## Validation-driven repairs

The broad gate was allowed to falsify implementation assumptions rather than being treated as a formality:

- Task `137` found the typed machine-contract inventory test still expected 16 contracted surfaces after the two new report contracts brought the checked cohort to 18. The named inventory expectation was updated and rerun.
- Task `139` found an over-strong accounting invariant during the interrupted-publication retry: all content blobs were reused, but the two verified manifest writes still transferred bytes. The validator now enforces the named schema bounds without falsely equating blob-upload count with total manifest-plus-blob transfer bytes.
- Security review added rejection for registry upload `Location` values that already contain a `digest` query, preventing ambiguous digest finalization.

Pueue task `142` reran the final state after those repairs. Exact key results were:

```text
registry core: 7 passed; 0 failed
registry shell: 2 passed; 0 failed
registry public CLI: 4 passed; 0 failed
machine schema contract generation: PASS (18 contracted, 47 classified)
machine schema contract check: PASS (18 contracted, 47 classified)
machine schema contract self-test: PASS
typed machine contracts: 4 passed; 0 failed
first-party root binary tests: 1545 passed; 0 failed
advisories ok, bans ok, licenses ok, sources ok
```

Task `142` also passed final Rustfmt, the configured Tiger Style rail, the complete serialized first-party quality gate, and `git diff --check`. The dependency command was the documented policy entry point:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
```

Its warnings remain the checked policy's reviewed duplicate/yanked/license inventory; the command exited successfully with all four policy classes `ok`.

## Pre-sync traceability

Task `130` ran the canonical pre-sync command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .
```

It reported:

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

The new active requirements were not yet accepted at that point. Their root-package implementation/verification bridge will be added only after sync so pre-sync Tracey carries no dangling references.

## Lifecycle evidence before sync

Pueue task `157` ran current Cairn validation plus proposal, design, and tasks gates over the staged implementation and evidence. Validation returned `valid: true`; every gate returned `valid: true` and `verdict: "PASS"`. The task packet reported 10 substantive tasks, 9 complete, and only the sync/Tracey/archive closeout task remaining.

## Accepted requirement and pre-archive packet

Pueue task `161` ran sync dry-run and execute for `publish-registry-backed-oci-workflow`. The execute plan had no reasons and receipt hash `fcabbb0051b84d01be220cff38de77d891c16f7ccff576942aca412f79ddb566`. Inspection confirmed all four reviewed IDs and their scenarios landed in `cairn/specs/kernel-bundle-oci/spec.md`:

```text
r[kernel_bundle_oci.registry_transport]
r[kernel_bundle_oci.registry_admission]
r[kernel_bundle_oci.registry_receipts]
r[kernel_bundle_oci.registry_verification]
```

After sync, `tools/tracey_refs.rs` linked implementation to the pure registry core, bounded shell, ordinary OCI admission, and public CLI; verification links to the core/shell, authenticated in-process registry, machine-contract, gallery, and documentation-drift rails. Pueue task `168` reran Cairn validation and all three gates; validation was `valid: true` and every verdict was `PASS`. Tracey reported:

```text
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

The stable profile total is consistent with the profile's accepted-spec selection; the run reported no missing or dangling reference.

## Claim boundary

This evidence proves only the reviewed explicit-bearer/anonymous Distribution endpoint subset and deterministic in-process registry behavior. It does not prove registry trust, authorization, credential validity beyond the tested exchange, tag immutability, signatures, transparency, exactly-once publication, transactional rollback, upload resumption, arbitrary-registry compatibility, kernel compatibility, bootability, deployability, release eligibility, or offline self-build completeness.
