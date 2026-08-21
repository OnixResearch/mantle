# V29 GNU binutils middle-sed audit repair

## Goal

Run the source-built proof with the complete V28 lower-sed audit repair and
retain the next exact StageX observation.

## V29 authority

- Source commit: `9476dd10`.
- Release orchestrator BLAKE3:
  `90af2db63a64a3be8ac018d8d893a2f8e7090b52a6acc014538cb87bec08a746`.
- Refreshed profile BLAKE3:
  `630e7c034f891cb49cf22df0802c66d7678efa43800d150df5286edd92ffc3e3`.
- Replacement Mantle source BLAKE3:
  `af7f6cab96642499aee1685a86cb9ca2407d89c68e7b70b730bb55435ef54aed`.
- Profile verification: Ready with zero missing, stale, unsupported, or
  untrusted records.
- The required root tool path is present.

## Result

V29 completed GNU binutils construction under protected execution. It then
failed the sed-specific total-event floor:

```text
bounds [73980, 74066], sed bounds [74008, 74066], observed 74001
```

The inventory records the already accepted middle sed form with `4,892`
invocations. The complete audit has `76,549` events. The trailing GNU binutils
suffix has `74,001` events.

## Repair

The middle-sed lower floor changes from `74,008` to `74,001`. The upper floor
remains `74,066`. No identity, executable digest, source, policy decision, or
fallback rule changes.

## Full preserved-audit validation

One current test binary replayed both retained forms:

```text
preserved-binutils-audit: events=73980 sed_invocations=4891
preserved-binutils-audit: events=74001 sed_invocations=4892
```

Both passed the ordinary production validators for all 68 identities, exact and
bounded counts, sed-derived counts, predecessor counts, and producer ordering.
See `preserved-audit-replays.log`.

The V29 audit BLAKE3 is
`5e37222bb655cfd8c39b6e5dd0372edfb00120d2ab7932a1298056bf472581b0`.
The inventory BLAKE3 is
`28e50356f3b69f276356e5f3e4569819419a9b57986bf757757c8e1b8eb8cfac`.

## Validation

- closed event-bound test: passed;
- positive and negative `mkdir` tests: passed;
- complete V28 and V29 audit replays: passed;
- isolated focused Clippy: passed;
- changed-file formatting and `git diff --check`: passed.

See `local-validation.log` and `preserved-audit-replays.log`.

## Non-claims

V29 stopped before provider construction, Rust bootstrap, stage1, stage2, or
receipt creation. A fresh proof remains required. This repair does not admit a
new executable, denied event, fallback path, or broader StageX behavior.
