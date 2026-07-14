# Adversarial review checkpoint

**Question:** Does the completed workspace implementation have a concrete path to cross-mode host leakage, unfenced mutation, quota bypass, snapshot mutation, mutable strong-cache reuse/publication, or clean-comparison relabeling?

**Inspected evidence:** `workspace.rs`, `workspace_shell.rs`, derivation/request transport, bwrap mount construction, orchestration cache lookup, remote registration/lease persistence and executor binding, pipeline/build reports, focused positive/negative tests, and the local secondary review findings.

**Decision:** No new implementation blocker was confirmed. The secondary review's proposed blockers were checked against the code:

- Mutable symlinks are observed without following; absolute and escaping relative targets quarantine before mount. Immutable roots must be declared castore inputs, mount read-only, reject a symlink root, and expose no arbitrary host path.
- Quotas are recomputed from bounded filesystem scans after restart rather than trusted counters; retention preserves active records and evicts deterministic idle/quarantine candidates.
- Snapshot ingestion copies the validated tree into castore and derives a content-addressed manifest ref.
- Mutable cache lookup is intentionally bypassed; reversing that bypass would violate the claim boundary. Remote mutable jobs also receive nonce-bound non-shared keys and no live output claims.
- Remote binding derives worker, authority, job, attempt, and fence from durable coordinator state, validates the current production attempt, injects the lease only after derivation identity verification, and releases it after execution.
- Clean comparison executes without a workspace, compares output sets, records separate digests, and leaves the original execution non-hermetic and non-publishable.

**Owner:** Mantle build/runtime maintainers.

**Next action:** Keep the repository-wide Clippy and Tracey blockers visible in `validation-transcript.txt`; no workspace code change is justified by the unconfirmed review suggestions. Add kernel-assisted beneath-root primitives only as future hardening if the workspace root becomes writable by untrusted host processes outside the sandbox lease.
