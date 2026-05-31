# V4 OpenSpec gates

Task-ID: V4
Covers: build.engine.dynamic.plans.abi, build.engine.dynamic.plans.declared.outputs, build.engine.dynamic.plans.scheduler, build.engine.dynamic.plans.provenance, defaults.dynamic.derivations
Date: 2026-05-31T06:05:52Z

## openspec validate

Command:

```sh
openspec validate native-dynamic-plans
```

Output:

```text
Change 'native-dynamic-plans' is valid

exit status: 0
```

## proposal gate

Command:

```text
openspec_gate stage=proposal change=native-dynamic-plans
```

Output excerpt:

```text
OpenSpec proposal gate (native-dynamic-plans)
VERDICT: PASS
Findings
- None
```

## design gate

Command:

```text
openspec_gate stage=design change=native-dynamic-plans
```

Output excerpt:

```text
OpenSpec design gate (native-dynamic-plans)
VERDICT: PASS
```

Design gate also emitted one info finding about not having a requirement-ID traceability table. It was not blocking.

## tasks gate

Command:

```text
openspec_gate stage=tasks change=native-dynamic-plans
```

Output excerpt:

```text
OpenSpec tasks gate (native-dynamic-plans)
VERDICT: PASS
Findings
- None
Stage Check
- Tasks are complete: `tasks.md` summary reports `16 done, 0 in progress, 0 todo`.
- Traceability is present: checkbox tasks include `[covers=...]` mappings to `build.engine.dynamic.plans.abi`, `build.engine.dynamic.plans.declared.outputs`, `build.engine.dynamic.plans.scheduler`, `build.engine.dynamic.plans.provenance`, and `defaults.dynamic.derivations`.
- Verification is present: V1–V4 include focused ABI, worker, provenance, and OpenSpec gate validation tasks with evidence paths.
- Stage is ready to move past tasks. Since all tasks are done, sync/archive is now required.
```

Note: the first tasks-gate run before V4 was checked failed only because V4 itself was unchecked. After recording this evidence and marking V4 complete, the final tasks gate passed.
