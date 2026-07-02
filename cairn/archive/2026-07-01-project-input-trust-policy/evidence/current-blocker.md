# Current Blocker — Project input trust policy

Date: 2026-07-01

## Question

Can `project-input-trust-policy` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/project-input-trust-policy/evidence/implementation-validation-2026-07-01.md` records current positive/negative core tests, shell verifier tests, CLI refresh fixtures, root compile check, formatting, and Cairn validate/gate transcripts.
- The first supported verifier kind is explicit and bounded: local Ed25519 detached signatures over the trust signature payload for the bound source digest. The implementation intentionally does not perform key-server lookup or forge trust.
- Refresh-time lock writes now require accepted trust facts for inputs and patches that declare trust policy; invalid or detached evidence fails before writing new lock data.
- Reports and locked trust records carry the bounded non-claim text from `PROJECT_INPUT_TRUST_NON_CLAIM`.

## Decision

No current blocker for the implemented scope. The remaining work is lifecycle cleanup: ensure tasks are checked, rerun Cairn validation after task/evidence edits, then sync/archive when ready.

## Owner

Mantle verification/project workflow owner.

## Next action

1. check completed tasks in `tasks.md`;
2. rerun Cairn validate/gates after evidence/task edits;
3. archive after the implementation commit is ready.
