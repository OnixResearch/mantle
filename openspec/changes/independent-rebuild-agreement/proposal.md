# Independent rebuild agreement

## Why

crunch can package a self-hosting proof and accept witness attestations, but the
stronger release claim needs explicit agreement from independent rebuilders.
The current workflow can show a matching witness; it does not yet define the
minimum independence evidence, agreement record, or verifier output needed to
say that independent rebuild agreement has been reached.

## What Changes

- **Define agreement records.** Add a canonical record that binds one release
  attestation to the set of accepted rebuild witnesses and their rebuilt output
  digests.
- **Strengthen independence inputs.** Require declared independence domains,
  environment summaries, and signer identities for witnesses that count.
- **Expose agreement verification.** Report when witness material satisfies the
  configured independent agreement threshold, separate from raw technical match.
- **Add operator docs and tests.** Cover matching, duplicate, non-independent,
  and mismatched witness sets.

## Non-Goals

- Requiring public transparency logs or network witness discovery.
- Proving full-source bootstrap or bit-for-bit reproducible artifacts.
- Replacing existing verifier-local policy; this extends it with a clearer
  agreement class.

## Capabilities

### New Capabilities
- `release-independent-agreement`: canonical agreement record and verifier
  output for independent witness agreement.
- `release-witness-independence-evidence`: structured witness environment and
  independence metadata.

### Modified Capabilities
- `release-verification-policy`: distinguish technical digest agreement from
  policy-recognized independent agreement.

## Impact

- **Files**: `crates/crunch-attestation-core`, `crates/crunch-attestation`,
  `src/attest_cmd.rs`, release CLI tests, docs.
- **APIs**: new agreement report fields and maybe `crunch attest agreement-show`.
- **Dependencies**: no network service dependency.
- **Testing**: canonicalization tests, policy tests, CLI integration tests for
  independent and non-independent witness sets.

## Relationship to Other Changes

This change defines social/technical agreement. `bit-for-bit-reproducible-release-artifacts`
can consume the agreement result as evidence for reproducibility claims.

## How to validate

1. `openspec validate independent-rebuild-agreement --strict` passes.
2. Canonical agreement records are byte-stable.
3. Verifier rejects mismatched or non-independent witness sets.
4. Verifier reports a distinct independent-agreement class for satisfying sets.
