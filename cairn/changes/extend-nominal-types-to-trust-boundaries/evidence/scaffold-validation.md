# Nominal trust-boundary scaffold validation

## Scope

This evidence records lifecycle scaffolding only. Implementation tasks remain unchecked. It does not prove any nominal-type migration or product behavior.

## Change creation

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- change create extend-nominal-types-to-trust-boundaries --root .
```

Result:

```text
{
  "change": "extend-nominal-types-to-trust-boundaries",
  "command": "change_create",
  "created": [
    "cairn/changes/extend-nominal-types-to-trust-boundaries",
    "cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
    "cairn/changes/extend-nominal-types-to-trust-boundaries/design.md",
    "cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
    "cairn/changes/extend-nominal-types-to-trust-boundaries/specs"
  ],
  "existing": [],
  "layout": "cairn",
  "mutated": true,
  "root": "."
}
```

## Dependency metadata

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- change depend add extend-nominal-types-to-trust-boundaries harden-remote-credential-boundary --root . --execute
```

Result tail:

```text
    "kind": "change_dependency_metadata",
    "manifest_hash": "d9bddcc768dd971b63d18ac755072dd0b231364afe0308dded420b5fbdba0c29"
  },
  "receipt_hash": "7026e32deb8427d476a22d6dabe985325e9f7a7174ca7339b6f0de5ded717bf1",
  "root": ".",
  "valid": true
}
```

The command created `metadata.json` with one dependency on `harden-remote-credential-boundary`.

## Baseline validation blocker

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
```

Result:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

The remote-builder warning that preceded this result was non-blocking. Cairn validation did not inspect lifecycle content because policy parsing failed first.

## Final default-policy validation

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
```

Result:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

## Decision

The change remains active with all implementation tasks unchecked. Refresh and review the generated Cairn policy before claiming default-policy validation.
