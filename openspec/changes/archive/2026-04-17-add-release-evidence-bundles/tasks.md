# Tasks: Add release evidence bundles

## Phase 1: Bundle format and commands

- [x] Define the release evidence manifest schema and required artifact set,
      including the staged-source archive, release binary artifact or artifacts,
      proof bundle, prerequisite inventory, workflow version or command
      identity provenance, the canonical manifest encoding rules, and the full-
      proof fields or markers required to reject prerequisite-only `--check`
      output deterministically.
- [x] Add CLI plumbing to create a release evidence bundle from local build and
      proof outputs without depending on extra verifier-side inputs.
- [x] Add CLI plumbing to verify an existing release evidence bundle.

## Phase 2: Artifact capture and claim boundaries

- [x] Capture source digest, binary digest, proof-bundle digest, and
      prerequisite-inventory digest in the bundle manifest.
- [x] Define and test that the staged-source archive bundled for release
      evidence is exported from the current tracked worktree, not a stale
      `HEAD` archive.
- [x] Record bounded claim metadata in the manifest so release evidence stays
      packaged integrity and proof-context evidence rather than a broader
      bootstrap claim.
- [x] Bind the bundled proof artifact and prerequisite inventory to the same
      release identifier, source digest, and binary digests recorded in the
      manifest by creating and validating the internal linkage record described
      in the design.
- [x] Ensure bundle creation copies the required proof artifacts instead of
      depending on ephemeral local paths only.
- [x] Reject prerequisite-only proof inputs such as `--check` output when
      building a release evidence bundle.
- [x] Reject bundles whose verification path still carries prerequisite-only
      `--check` proof evidence instead of a full proof artifact.
- [x] Update README and bootstrap-facing docs so release evidence is described
      as packaged integrity evidence, not as automatic full-source bootstrap or
      global reproducibility proof.

## Phase 3: Validation coverage

- [x] Add tests that bundle creation fails when required artifacts are missing
      and names the missing required artifact.
- [x] Add tests that `crunch release verify` fails when the bundle omits any
      required member such as the staged-source archive, release binary,
      proof artifact, or prerequisite inventory.
- [x] Add tests that bundle verification fails on digest mismatch, manifest
      schema mismatch, proof-linkage mismatch, prerequisite-inventory linkage
      mismatch, missing workflow provenance, or non-canonical manifest
      encoding, and identifies the mismatched artifact.
- [x] Add tests that manifest claim-boundary violations are rejected and that
      docs do not over-claim what bundle verification proves.
- [x] Add tests that a prerequisite-only proof artifact is rejected for release
      evidence generation and for verification, and says that a full proof
      artifact is required.
- [x] Add tests that `crunch release verify` succeeds or fails using only
      bundle-local contents, with no external proof, source, or build inputs.
- [x] Add tests that a valid bundle verifies successfully and reports the
      bundled release identifier and artifact digests.

## Validation

- [x] Produce a full proof artifact with `./scripts/prove-self-hosting.sh`
      using the repo's documented environment, create a release evidence bundle
      from that artifact, then run `crunch release verify` against the produced
      bundle using bundle-local contents only and keep the command output, proof
      bundle path, stage diagnostics, and `test result:` lines. Do not treat
      `--check` output as sufficient proof evidence.
  - Proof evidence: pueue task `69` (`release-evidence-full-proof-rerun4`)
    finished `Success` with `test result: ok. 1 passed; 0 failed; 0 ignored; 0
    measured; 41 filtered out; finished in 3079.20s`.
  - Proof bundle: `target/self-hosting-proof/release-evidence-fresh5`
    (`manifest.json`, `summary.txt`, `stage0/diagnostics.txt`,
    `stage2/diagnostics.txt`).
  - Release bundle command output kept from:
    `cargo --config .cargo/vendor-config.toml run --quiet -- release create --release-id release-evidence-fresh5 --binary target/self-hosting-proof/work-release-evidence-fix5/tmp/crunch-builds/a5493eed-3b36-4896-b3c1-c2c70076cd73-jJYTCV/scratches/nix/store/dq2za2qwiz8v9dwrx8iz62xajsfbfq66-crunch/bin/crunch --proof-bundle target/self-hosting-proof/release-evidence-fresh5`
  - Bundle-local verification output kept from:
    `cargo --config .cargo/vendor-config.toml run --quiet -- release verify target/release-evidence/release-evidence-fresh5`
- [x] Run `openspec validate add-release-evidence-bundles`.
