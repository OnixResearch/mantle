# Proposal: Project input trust policy

## Summary

Add per-input and per-patch trust checks so Mantle can reject lockfile refreshes when fetched source material lacks required signatures, signer identities, or digest-bound provenance.

## Motivation

Nixtamal's roadmap includes manifest `trust` keys for minisign or PGP checks. Mantle already has attestation and release-verification concepts, so project inputs should not stop at content hashes when an upstream provides signed releases, signed patch files, or pinned public keys.

Trust policy must be explicit and evidence-backed. It should reject bad or missing signatures before updating the lockfile, while keeping content hash verification as the baseline integrity check.

## Scope

- Define manifest trust policy for project inputs and patch definitions.
- Support signature material references, trusted public key identities, required signer sets, and digest binding.
- Record trust-check results in lock update reports and project attestations where applicable.
- Fail closed when required trust material is missing, malformed, unsigned, signed by an untrusted key, or detached from the fetched bytes.
- Keep trust verification in the shell/adapter layer with pure core decisions over verified trust facts.

## Non-goals

- No automatic trust in key servers or host forges.
- No replacement for content hashes.
- No claim that input trust proves build reproducibility, compiler correctness, or release validity.

## Target Spec Domains

- `project-workflows` for input and patch trust policy.
- `verification-evidence` for bounded claims and evidence wording if trust results become release-facing.
