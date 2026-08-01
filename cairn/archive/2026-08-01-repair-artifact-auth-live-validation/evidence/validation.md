# Validation evidence

## Focused gate

Command:

```text
nix build .#checks.x86_64-linux.artifact-auth-radicle-cutover --no-link -L
```

Pueue task `7419` completed with status `Done`:

```text
this derivation will be built:
  /nix/store/dx9l4h12z2krblbbfc3fywnsvn20anc1-mantle-artifact-auth-radicle-cutover.drv
cannot build on 'ssh-ng://root@10.10.10.1': error: failed to start SSH connection to '10.10.10.1'
building '/nix/store/dx9l4h12z2krblbbfc3fywnsvn20anc1-mantle-artifact-auth-radicle-cutover.drv'...
```

The remote-builder warning did not block the local successful build. The check ran the current, unrelated-comment, and wrong-revision fixtures.

## Historical evidence preservation

Command:

```text
git diff --exit-code origin/main -- evidence/radicle/artifact-auth-cutover-v1.ncl evidence/radicle/artifact-auth-cutover-v1.json evidence/radicle/artifact-auth-cutover-v1.blake3
git diff --check
```

Pueue task `7431` completed with status `Done` and no command output. The historical receipt files are unchanged.

## Formatting

Command:

```text
nixfmt --check flake.nix
```

Pueue task `7430` completed with status `Done` and no command output.

## Lifecycle gates

The repository policy predates Cairn's required `nominal_identity_policy`. The current sibling Cairn policy was selected explicitly, as existing Mantle lifecycle evidence requires.

- Pueue task `7438`: Cairn validation returned `"valid": true`.
- Pueue task `7439`: proposal gate returned `"valid": true` and `"verdict": "PASS"`.
- Pueue task `7441`: design gate returned `"valid": true` and `"verdict": "PASS"`.
- Pueue task `7440`: tasks gate returned `"valid": true` and `"verdict": "PASS"`.

## Broad validation blockers

Pueue task `7402` ran current-policy Tracey coverage before sync. It reported:

```text
"profile_id": "cairn-default"
"referenced": 253
"requirements": 680
"valid": false
"verdict": "fail"
error: tracey coverage failed
```

This repository-wide debt predates this change. The change adds a `tools/tracey_refs.rs` bridge for its new requirement.

After sync, pueue task `7475` reported `681` requirements and `254` referenced requirements. The new requirement increased both counts by one. Its ID was absent from both `missing` and `dangling`. The broad pre-existing coverage debt kept the final verdict at `fail`.

Pueue task `7435` ran `nix flake check -L`. The repaired artifact-auth check passed. The broad gate stopped at an unrelated baseline blocker:

```text
bootstrap blocker inventory: 115 findings across 3 classes, 355 evidence-backed suppressions, 0 promotion claims, enforce=true
FAIL: bootstrap blocker inventory is not clean; expected 0 findings and 0 promotion claims
```

## Claim boundary

The focused artifact-auth gate passes. The broad Tracey and bootstrap-inventory gates are not clean. This change does not claim repository-wide release readiness.
