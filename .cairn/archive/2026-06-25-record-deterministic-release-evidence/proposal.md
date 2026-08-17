# record deterministic release evidence

## Problem

The provider fixed-point release bundle now has real deterministic-release proof output, but the durable repository record is still incomplete. The proof receipts and summaries live under generated `target/` paths that must remain untracked, so future release-readiness or status claims could otherwise depend on ephemeral local artifacts or chat transcript output.

Mantle needs a tracked evidence transcript that records the exact bounded claim: the packaged stage2 Mantle artifact rebuilt twice from recorded release inputs under recorded sandbox profiles, the BLAKE3 digest sets matched, and the bundled provider fixed-point proof remained valid. The transcript must preserve non-claims and avoid checking in large generated proof material.

## Proposed change

Add a tracked Cairn evidence transcript for the `provider-fixed-point-release-evidence-2026-06-25` bundle. The transcript should record the release reproduce output, required verifier output, receipt-checker output, key digests, and the clean tracked worktree status after evidence capture.

Keep the generated release bundle, rebuild outputs, deterministic proof receipts, and summaries under ignored `target/` paths. The tracked artifact is the concise evidence transcript only.

## Success criteria

- A tracked evidence transcript names the release bundle and exact deterministic/provider proof commands that were run.
- The transcript records the reproducibility report digest, deterministic proof digest, sandbox isolation evidence digest, provider proof status, and artifact digest.
- The transcript states the bounded non-claim: packaged-artifact determinism, not full bootstrap reproducibility or full Cargo compatibility.
- Generated `target/` artifacts remain untracked.
- Cairn validation remains clean after the transcript is committed.
