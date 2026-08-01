# Lifecycle gate status

Date: 2026-08-01

## Final validation attempt

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

The current sibling Cairn binary stops before it reads the change package. Proposal, design, tasks, and Tracey gates therefore did not run.

The earlier non-mutating policy refresh probe also failed:

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- policy export --root . --output /tmp/mantle-cairn-policy-20260801.json
error: Nickel import must be repository-root-relative
```

The active `extend-nominal-types-to-trust-boundaries` change owns the missing nominal identity policy surface. This closure change does not modify that unrelated active policy work.

V4 remains incomplete. This change must not sync or archive until the policy owner repairs the generated policy and all lifecycle gates pass.
