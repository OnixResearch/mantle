## Phase 1: Manifest schema and release creation

- [x] [serial] Add Git source acquisition metadata to the release evidence core. r[verification_evidence.git_source_witness_replay]
  - Positive test: a Git source acquisition record with supported URL, commit, archive profile, and matching BLAKE3 archive digest canonicalizes and verifies.
  - Negative tests: malformed commit, empty remote URL, credential-bearing URL, unsupported scheme, and archive digest mismatch are rejected.
- [x] [serial] Add release creation CLI/request plumbing for Git source-origin metadata. r[verification_evidence.git_source_witness_replay]
  - Positive test: `mantle release create` records Git remote/ref/commit metadata and the generated source archive digest.
  - Negative test: Git-origin flags that omit a commit or conflict with external-archive flags fail before writing a manifest.

## Phase 2: Deterministic Git archive reconstruction

- [x] [serial] Extract release source archive membership/order rules into a pure deterministic core. r[verification_evidence.git_source_witness_replay]
  - Positive test: unordered in-memory Git tree entries produce stable archive member order and digest inputs.
  - Negative tests: parent-directory paths, absolute paths, symlinks outside the source tree, submodules, and private runtime paths are rejected or excluded according to policy.
- [x] [serial] Implement the Git fetch/checkout/archive shell around the deterministic core. r[verification_evidence.git_source_witness_replay]
  - Positive test: a local `file://` bare repo at a pinned commit reconstructs the same archive digest as release creation.
  - Negative tests: wrong commit, missing ref, dirty checkout assumptions, and generated archive digest mismatch fail closed.

## Phase 3: Witness strict Git-source gate

- [x] [serial] Add `mantle release witness-rebuild --require-git-source` (or equivalent strict mode) and plan-time validation. r[verification_evidence.git_source_witness_replay]
  - Positive test: a request with Git source metadata plans Git-source replay.
  - Negative test: requiring Git source on copied-source or external-archive-only requests fails before workflow launch.
- [x] [serial] Use witness-generated Git archive bytes for extraction and workflow launch. r[verification_evidence.git_source_witness_replay]
  - Positive test: a local Git fixture source archive is generated, verified, extracted, and used instead of publisher-supplied archive bytes.
  - Negative test: digest mismatch records expected/generated digests and does not extract or launch.
- [x] [serial] Record Git source derivation in witness audit metadata. r[verification_evidence.git_source_witness_replay]
  - Positive test: successful audit names Git mode, remote URL, commit, ref/tag policy, generated digest, generated archive path, and status.
  - Negative test: Git verification failure audit records the failing source stage without leaking credentials.

## Phase 4: Cross-machine proof and lifecycle evidence

- [x] [serial] Run focused core, release evidence, and witness rebuild tests. r[verification_evidence.git_source_witness_replay]
  - Evidence: record exact command output under this change's evidence directory.
- [x] [serial] Run Cairn validation and proposal/design/tasks gates. r[verification_evidence.git_source_witness_replay]
  - Evidence: record `cairn validate`, `cairn gate proposal`, `cairn gate design`, and `cairn gate tasks` output.
- [ ] [serial] Run Aspen1 Git-source witness replay and final verification. r[verification_evidence.git_source_witness_replay]
  - Evidence: record source remote/ref/commit, generated source digest, witness audit, rebuilt output digest, witness key, and final `quorum-satisfied` verification.
  - Blocked in this environment: `ssh -o BatchMode=yes -o ConnectTimeout=5 aspen1 true` failed with `Could not resolve hostname aspen1`; see `evidence/bounded-claims-and-blockers-2026-06-29.md`.
- [x] [serial] Record bounded claims and non-claims. r[verification_evidence.git_source_witness_replay]
  - Evidence: state that this proves Git-derived source archive replay for the exact release and policy, not compiler correctness, generic hosting API correctness, or deploy success.
