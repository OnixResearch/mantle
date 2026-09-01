# Action trust promotion scope validation — 2026-08-01

Status: lifecycle scope update only. This evidence does not claim bootstrap promotion or release readiness.

## Goal

Require parity promotion to consume and independently review the fixed-point root action trust evidence. Promotion must reject partial action lists, path-only generated authority, producerless executables, unmatched events, digest drift, producer drift, count drift, remote execution, and cache-only completion.

## Oracle checkpoint

| Field | Value |
|---|---|
| Question | Where must independent review of the full action list occur? |
| Inspected evidence | The fixed-point v2 receipt contract, StageX protected-exec receipts, bootstrap parity requirements, promotion checker tasks, and the upstream root action-audit design. |
| Decision | The fixed-point change creates and binds the evidence. The promotion change independently recomputes links, counts, and planned-versus-observed coverage. |
| Owner | Promotion tasks I4 through I6 and verification tasks V1 through V3. |
| Next action | Extend the standalone promotion verifier after the fixed-point action-trust schema exists. |

## Policy selection

The repository-local generated policy lacks Cairn's required `nominal_identity_policy`. The active `extend-nominal-types-to-trust-boundaries` change owns that refresh.

Validation selected this current sibling policy explicitly:

```text
/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

## Validation

Pueue task `7352` ran repository validation plus proposal, design, and tasks gates for both affected changes. The ordered command chain exited successfully. The final promotion tasks gate recorded:

```json
{
  "valid": true,
  "verdict": "PASS"
}
```

The exact command set is preserved in the fixed-point evidence file:

```text
cairn/changes/prove-source-built-mantle-fixed-point/evidence/action-trust-scope-validation-2026-08-01.md
```

Pueue task `7347` ran `git diff --check` successfully. Final pueue task `7354` repeated `git diff --check`, repository validation, and all six targeted gates after both evidence files existed. The ordered command chain exited successfully.

## Non-claims

This update does not prove that action-trust evidence exists, that the independent verifier accepts it, that any parity axis is complete, or that Mantle is release-ready.
