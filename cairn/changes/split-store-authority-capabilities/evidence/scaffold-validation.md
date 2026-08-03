# Scaffold validation

This package is an active design scaffold. All implementation and final-validation tasks remain unchecked.

## Repo-local policy blocker

The default command was run first:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
```

It stopped before lifecycle validation with this existing policy diagnostic:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

This change does not modify or regenerate the shared policy.

## Explicit current-policy validation

Pueue task `7647` ran the following command chain under `set -eu` with the current Cairn-generated policy:

```text
POLICY=/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal split-store-authority-capabilities --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design split-store-authority-capabilities --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks split-store-authority-capabilities --root . --policy "$POLICY"
git diff --check
```

The complete chained command exited successfully. The final Cairn result reported:

```text
"valid": true,
"verdict": "PASS"
```

This proves scaffold structure and gate acceptance only. It does not prove implementation, tests, behavior, or completion.
