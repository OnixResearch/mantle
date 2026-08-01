# Scaffold validation

Date: 2026-08-01

## Attempted command

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

The SSH remote-builder warning was non-blocking. Validation stopped before proposal, design, or tasks gates.

## Refresh probe

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- policy export --root . --output /tmp/mantle-cairn-policy-20260801.json
error: Nickel import must be repository-root-relative
```

The current sibling Cairn binary cannot validate this Mantle checkout until the pre-existing policy compatibility blocker is repaired. This change does not modify the unrelated generated policy or its Nickel imports.

The change package remains a scaffold until the gates run successfully. No Cairn validation claim is made.
