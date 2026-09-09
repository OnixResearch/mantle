# Scaffold validation

This package is an active design scaffold. All implementation and final-validation tasks remain unchecked.

## Repo-local policy blocker

The default repository validation stopped before lifecycle validation because `cairn-policy/generated/cairn-policy.json` lacks the current required `nominal_identity_policy` field. The exact diagnostic was:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

This change does not modify or regenerate the shared policy.

## Explicit current-policy validation

Pueue task `7647` ran repository validation plus proposal, design, and tasks gates for this package under `set -eu` and with:

```text
--policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

It also ran `git diff --check`. The complete chained command exited successfully. The final Cairn result reported:

```text
"valid": true,
"verdict": "PASS"
```

This proves scaffold structure and gate acceptance only. It does not prove implementation, tests, behavior, or completion.
