# Scaffold validation

Date: 2026-08-01

## Authoritative Mantle policy attempt

The repository-default commands did not reach artifact validation. Mantle’s generated policy predates the current Cairn schema.

Commands:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal backport-snix-correctness-fixes --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design backport-snix-correctness-fixes --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks backport-snix-correctness-fixes --root .
```

Pueue tasks: `7395`, `7397`, `7396`, and `7398`.

Each command ended with this policy error:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

This is a repository-wide pre-existing blocker. This change does not modify the generated policy or its Nickel source.

## Structural validation with Cairn’s current default policy

The following commands used Cairn’s current generated policy only to inspect lifecycle structure. This policy override is not a substitute for final validation with Mantle’s policy.

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal backport-snix-correctness-fixes --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design backport-snix-correctness-fixes --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks backport-snix-correctness-fixes --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Results:

- Pueue task `7404`: validation ended with `"valid": true`.
- Pueue task `7403`: proposal gate ended with `"valid": true` and `"verdict": "PASS"`.
- Pueue task `7405`: design gate ended with `"valid": true` and `"verdict": "PASS"`.
- Pueue task `7402`: tasks gate ended with `"valid": true` and `"verdict": "PASS"`.

The tasks gate reported 25 substantive tasks, 2 completed planning tasks, and 23 implementation or validation tasks still open.

## Decision

The Cairn scaffold is structurally valid under the current default Cairn policy. Final Mantle-policy validation remains blocked until the repository-owned generated policy is refreshed through its Nickel source and export workflow.

## Owner and next action

The Mantle lifecycle-policy owner must refresh `cairn-policy/generated/cairn-policy.json` from the repository-owned policy source. The Snix change owner must then rerun V4 without a policy override.
