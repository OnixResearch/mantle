# Root action trust scope validation — 2026-08-01

Status: lifecycle scope update only. This evidence does not claim implementation, fixed-point success, bootstrap promotion, or release readiness.

## Goal

Extend the active source-built fixed-point change with one complete action trust contract. The contract lists all actions before execution and reconciles them with observed execution afterward.

The update requires:

- fixed or producer-linked executable authority;
- explicit input and output authority;
- local-only execution for the proof;
- bounded event counts and resources;
- rejection of incomplete action adapters;
- v2 receipt links for the plan and observations;
- a derived `mantle --json bootstrap trust-report --proof-root <path>` operator view.

ADR 0070 records the architecture boundary. The README records the MIT-licensed `fzakaria/stage0-bazel` design reference. Mantle retains BLAKE3 identity, producer-linked authority, seccomp enforcement, and evidence authority.

## Oracle checkpoint

| Field | Value |
|---|---|
| Question | Which `stage0-bazel` action-audit ideas belong in the source-built Mantle proof? |
| Inspected evidence | StageX materialization plans and receipts, source-built fixed-point plans and receipts, protected-exec behavior, Rust unit plans, parity promotion artifacts, and the upstream action-audit and attestation rules. |
| Decision | Add one root-scoped action trust plan and planned-versus-observed reconciliation. Do not adopt path-prefix trust, host-shell exceptions, test exceptions, Bazel, or a separate evidence authority. |
| Owner | Fixed-point tasks I3 through I5 create and bind the evidence. |
| Next action | Implement the pure action-trust core and complete StageX, native-provider, and Rust-unit adapters under I3. |

## Policy selection

The repository-local generated policy predates Cairn's required `nominal_identity_policy`. The active `extend-nominal-types-to-trust-boundaries` change owns that refresh.

The first baseline command stopped with this exact blocker:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy; refresh_command=nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

The lifecycle runs therefore selected this current sibling Cairn policy explicitly:

```text
/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

## Validation

The baseline command in pueue task `7339` exited successfully before the scope update. The post-update command in pueue task `7352` also exited successfully.

Both tasks ran this ordered command set with the explicit policy above:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal prove-source-built-mantle-fixed-point --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design prove-source-built-mantle-fixed-point --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks prove-source-built-mantle-fixed-point --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal promote-full-bootstrap-parity --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design promote-full-bootstrap-parity --root . --policy "$POLICY"
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks promote-full-bootstrap-parity --root . --policy "$POLICY"
```

The command chain uses `&&`. A later command cannot run after an earlier failure. The final post-update tasks gate recorded:

```json
{
  "valid": true,
  "verdict": "PASS"
}
```

Pueue task `7347` ran `git diff --check` successfully. Final pueue task `7354` repeated `git diff --check`, repository validation, and all six targeted gates after both evidence files existed. The ordered command chain exited successfully.

## Non-claims

This update defines required work. It does not prove a complete action list, observed execution coverage, StageX success, native or Rust provider construction, matching Mantle binaries, compiler correctness, seed correctness, release reproducibility, or deployment success.
