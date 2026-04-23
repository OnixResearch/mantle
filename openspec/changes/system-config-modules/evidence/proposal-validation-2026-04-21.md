# Proposal validation evidence

## Command: `openspec validate system-config-modules`

```text
Change 'system-config-modules' is valid
```

## Command: `openspec_gate stage=proposal change=system-config-modules`

```text
VERDICT: FAIL

## Findings
- [high] [class=omission] [scope=spec] [route=spec-rule] [promoted=yes] Module output contract is assumed by later specs but never defined.
- [medium] [class=omission] [scope=spec] [route=spec-rule] [promoted=yes] Stable CLI/result contract lacks deterministic ordering rules for diagnostics.
- [medium] [class=omission] [scope=review] [route=human] [promoted=no] Proposal-stage validation evidence referenced by the proposal is not supplied.
```

## Status

This file records the latest rerun transcript while the proposal gate is still
failing. Update it again once the gate reaches `VERDICT: PASS`.
