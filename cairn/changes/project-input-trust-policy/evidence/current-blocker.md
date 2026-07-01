# Current Blocker — Project input trust policy

Date: 2026-06-30

## Question

Can `project-input-trust-policy` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/project-input-trust-policy/tasks.md` still has 9 unchecked tasks covering input/patch trust schema, evidence wording, pure policy validation, shell trust verification, refresh/patch/project-attestation integration, and positive/negative/shell tests.
- The specs require supported verifier kinds, signature refs, trusted key identities/fingerprints, required signer or quorum decisions, digest binding, and bounded claim wording in `verification-evidence`.
- Current Mantle has release/signature and attestation concepts, but code search did not find a project input trust-policy model, minisign/PGP verifier path, or refresh-time lock update gate for input/patch trust evidence.
- The proposal explicitly excludes key-server/forge trust; selecting the first verifier formats and fixture strategy is a product/API decision that must be made before support can be claimed.

## Decision

Blocked. The change needs an explicit verifier support decision and refresh/attestation integration. Draining it now would overclaim signed/trusted input evidence from ordinary content hashes.

## Owner

Mantle verification/project workflow owner for refresh-time input trust evidence.

## Next action

1. decide the first verifier kind and local fixture format, keeping key-server/forge lookup out of scope;
2. implement pure trust-policy decisions over explicit verified facts;
3. add shell verification for local fixture keys/signatures and fail-closed refresh behavior;
4. project only bounded trust claims into reports/attestations with positive and negative tests.
