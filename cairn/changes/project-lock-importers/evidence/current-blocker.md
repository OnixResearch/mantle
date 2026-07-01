# Current Blocker — Project lock importers

Date: 2026-06-30

## Question

Can `project-lock-importers` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/project-lock-importers/tasks.md` still has 9 unchecked tasks covering external importer contracts, Nixtamal mapping, pure conversion, shell parsing/apply surfaces, future adapter seams, and positive/negative/CLI tests.
- `proposal.md`, `design.md`, and `specs/project-workflows/spec.md` require no-mutate plan/apply behavior and Nixtamal mapping for source kinds, mirrors, patches, BLAKE3, frozen inputs, freshness, fetch policy, and trust policy.
- The active prerequisite packages `project-freshness-probes`, `input-fetch-policy`, `project-input-trust-policy`, and `forge-agnostic-vcs-inputs` are still unchecked and unimplemented, so an importer cannot preserve those semantics without either fabricating support or silently dropping data.
- Current code search did not find a Mantle Nixtamal/importer implementation surface; existing project code remains centered on current manifest/lock refresh, mirrors, patches, generated inputs, and attestation adapters.
- `README.md` has an uncommitted reference to Nixtamal as prior art; that reference is useful context, not implementation evidence.

## Decision

Blocked. The package is a valid design scaffold, but draining it now would overclaim importer support before the source/freshness/fetch/trust/VCS semantics it must preserve exist in Mantle.

## Owner

Mantle project workflow/importer owner after the dependent project-input semantics land or the importer scope is reduced.

## Next action

Implement or split prerequisites first:

1. land bounded project freshness, fetch-policy, trust-policy, and non-Git VCS input semantics, or explicitly scope the first importer to data that exists today;
2. define the normalized `ExternalPinSet` core model and blocker taxonomy;
3. add a Nixtamal fixture format/parser decision with positive and negative fixtures;
4. implement no-mutate plan, reviewed apply, and CLI tests proving only planned Mantle-owned files are written.
