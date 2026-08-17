## Context

`create_release_evidence_bundle` prepares the requested public directory and then copies each artifact into it before constructing and writing `manifest.json`. Any error after the first copy leaves observable partial state. `prepare_output_bundle_dir` then rejects that nonempty directory on retry.

Assembly needs a transaction boundary: plan, stage, verify, and commit.

## Decisions

### Plan before mutation

A pure planner receives normalized input facts, final destination observations, and publication policy. It validates required inputs, destination shape, artifact naming, collisions, optional evidence dependencies, and a deterministic assembly order. The final destination must be absent for a no-clobber commit; an existing file, symlink, empty directory, or nonempty directory is a blocker.

The plan carries a BLAKE3 identity over release id, final relative artifact layout, input identities available before copying, and policy. Temporary path randomness is shell state and does not affect evidence identity.

### Stage in a private sibling directory

The shell creates a unique, mode-restricted staging directory beneath the final destination's real parent, ensuring staging and final paths share a filesystem. All bundle-relative operations use the staging capability root. The staging name and ownership marker identify Mantle-created state but are not release evidence.

Source, binaries, proof material, and optional evidence are copied according to the plan. The canonical manifest is written last, after all artifact records are complete.

### Verify before one no-clobber commit

The normal production bundle verifier runs against the staging root, including canonical manifest bytes, artifact hashes, proof linkage, and enabled external evidence checks available at creation. Only a fully verified stage is eligible for publication.

Publication uses a same-parent atomic no-replace rename. If another process creates the destination after planning, commit fails and leaves that destination untouched. The CLI reports creation success only after rename completes.

### Isolate failure and make retry safe

Before commit, any copy, serialization, validation, or verification failure leaves the final destination absent and returns a phase-specific diagnostic. The shell attempts bounded cleanup of the current staging root. If cleanup fails or the process is interrupted, a later invocation may quarantine or remove only a sibling carrying a valid Mantle ownership marker and matching plan identity; it never recursively deletes an arbitrary similarly named path.

Retry creates a fresh stage and does not require manual repair of the final path. Existing completed destinations remain no-clobber errors rather than implicit overwrite targets.

### Preserve core and shell boundaries

Planning, assembly-state transitions, failure classification, and commit eligibility are pure. Directory creation, capability opening, copying, syncing where supported, verification I/O, rename, cleanup, diagnostics, and output are shell responsibilities.

Named test failpoints are injected through a test-only shell adapter at phase boundaries; production logic does not depend on ambient failpoint environment variables.

## Risks / Trade-offs

- Callers that pre-create an empty output directory must stop doing so because atomic publication requires an absent final path.
- Standard-library rename semantics vary; the implementation must use a no-clobber primitive or a platform abstraction that preserves the contract.
- Atomic visibility does not by itself guarantee persistence across sudden power loss. Documentation must keep crash visibility distinct from storage durability.
- Safe stale-stage cleanup is intentionally conservative and may leave quarantined diagnostic state rather than delete uncertain paths.
