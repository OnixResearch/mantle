# Proposal validation

## Scope

This session created a proposal only. No implementation, producer build, dependency update, source publication, deployment, spec sync, or archive occurred.

The package remains local and uncommitted. Existing README changes, other proposals, and active proof outputs remain outside this change.

## Repository-policy blocker

The baseline command was:

```sh
cairn validate --root . --policy cairn-policy/generated/cairn-policy.json
```

The retained CLI rejected this repository policy before package creation:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: cairn.workflow.profile.registry at workflow_profile_policy.schemas: initial workflow profile is missing: outcome-machine
```

This result blocks repository-policy acceptance. The proposal does not change the policy or resolve the CLI/policy compatibility question.

## Secondary structural checks

The proposal, design, and tasks gates and whole-repository validation succeeded with the explicit shared Cairn policy:

```sh
C=/home/brittonr/git/OnixResearch/campaign/.pi/complete-20260906/cairn-forge-tool/bin/cairn
P=/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
"$C" gate proposal stabilize-spacewasm-bundle-evidence --root . --policy "$P"
"$C" gate design stabilize-spacewasm-bundle-evidence --root . --policy "$P"
"$C" gate tasks stabilize-spacewasm-bundle-evidence --root . --policy "$P"
"$C" validate --root . --policy "$P"
```

Receipts are `shared-policy-proposal.json`, `shared-policy-design.json`, `shared-policy-tasks.json`, and `shared-policy-validation.json` in this directory. They record the selected policy identity and input hashes.

These checks establish package structure under that policy only. They do not establish Mantle repository-policy acceptance, implemented behavior, reproducibility, or consumer admission. Every task remains unchecked.
