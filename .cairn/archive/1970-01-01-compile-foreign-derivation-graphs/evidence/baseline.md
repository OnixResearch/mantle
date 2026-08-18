# Pre-change baseline

Date: 2026-08-01
Branch: `agent/compile-foreign-derivation-graphs-7625`
Baseline commit: `36833dcb`

## Cairn gate preflight

The first gate run failed before artifact review because the checked Mantle policy lacks the current Cairn `nominal_identity_policy` field.

Commands:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal compile-foreign-derivation-graphs --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design compile-foreign-derivation-graphs --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks compile-foreign-derivation-graphs --root .
```

Pueue tasks: `7646`, `7645`, and `7647`.

Exact diagnostic:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

The documented refresh command also failed before writing output.

```text
error: policy missing field nominal_identity_policy
```

Pueue task: `7649`.

The current sibling Cairn policy was then selected explicitly. The active `extend-nominal-types-to-trust-boundaries` change owns the Mantle policy refresh.

```text
--policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

The read-only gate rerun passed:

```text
proposal: valid=true, verdict=PASS
design: valid=true, verdict=PASS
tasks: valid=true, verdict=PASS, task_done=0, task_todo=15
```

Pueue tasks: `7675`, `7674`, and `7676`.

## Focused Rust baseline

### Mantle foreign import core

Command:

```text
nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import
```

Pueue task: `7660`.

Exact result:

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1991 filtered out; finished in 0.00s
```

### crunch-glue

Command:

```text
nix develop -c cargo test -p crunch-glue
```

Pueue task: `7661`.

Exact results:

```text
test result: ok. 83 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Foreign import CLI

Command:

```text
nix develop -c cargo test -p mantle --test foreign_import_cli
```

Pueue task: `7662`.

Exact result:

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```
