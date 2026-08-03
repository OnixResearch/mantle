# Report Mantlepkgs base-to-head impact

## Why

Mantlepkgs generations expose package records and build evidence. Reviewers still lack one deterministic report that compares a base generation with a candidate generation.

Without this report, CI adapters must infer changed packages, result transitions, closure growth, and dependency retention from unrelated artifacts.

## What Changes

- Add a versioned `mantle-package-impact-v1` machine report.
- Compare compatible base and head catalog generations through a pure core.
- Report catalog dispositions and explicit build-outcome transitions.
- Report comparable closure member, byte, and retained-dependency deltas.
- Reuse admitted Mantle action results under existing authority rules.
- Keep GitHub, forge credentials, webhooks, approvals, and comments in external adapters.

## Non-Goals

- Treating a derivation path as sufficient action-result authority.
- Calling an unbuilt package broken or fixed.
- Comparing closures across incompatible systems, store prefixes, policies, or incomplete facts.
- Adding a hosted CI service to Mantle.
- Proving package correctness or release eligibility from a favorable diff.

## Dependencies

- `generate-mantlepkgs-from-nixpkgs` supplies versioned package records and catalog identities.
- `structure-mantlepkgs-domains` supplies explicit domains, variants, and validation roots.
- ADR 0024 preserves action-result admission outside raw CAS presence.

## Impact

- **Affected specs:** new `mantlepkgs-impact-evidence` specification.
- **Affected code:** pure impact core, Mantlepkgs CLI adapter, report schema, closure comparison adapter, and fixtures.
- **Affected evidence:** base and head identity bindings, package dispositions, outcome transitions, closure deltas, and non-comparability reasons.
- **Compatibility:** report schema changes require a new version. Existing build reports remain unchanged.
