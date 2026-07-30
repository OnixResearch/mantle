# Scaffold validation — 2026-07-30

## Status

This change is a validated lifecycle scaffold. All implementation and verification tasks remain unchecked.

No castore scan, executable classification, provenance decision, or audit receipt is implemented or proven by this transcript.

## Repository-policy blocker

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
```

Exact decisive output:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

The repository policy needs a separate reviewed refresh before it can validate with the current Cairn binary.

## Current-policy structural validation

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Exact decisive output:

```text
"valid": true
```

## Change gates

Commands:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal audit-foreign-realization-provenance --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design audit-foreign-realization-provenance --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks audit-foreign-realization-provenance --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Exact decisive output for each command:

```text
"valid": true,
"verdict": "PASS"
```

The tasks gate reported 14 unchecked tasks.

## Diff hygiene

Command:

```text
git diff --check
```

Result: exit status 0 with no diff diagnostics.
