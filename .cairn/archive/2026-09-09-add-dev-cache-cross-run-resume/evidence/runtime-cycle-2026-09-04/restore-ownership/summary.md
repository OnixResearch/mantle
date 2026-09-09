# Preserve pre-existing payloads after rejected restore validation

## Outcome and owner

The Mantle checkpoint shell now keeps the payload paths that each restore creates. Failed validation removes only those payloads. Reused dev-store payloads remain unchanged.

The shell also distinguishes rejected validation from failed cleanup. Failed cleanup stops the attempt. It cannot authorize cold fallback.

Mantle owns this behavior in `src/source_built_fixed_point_checkpoint_shell.rs` and its dev-resume integration. The Nix integration check now includes the checkpoint-shell tests.

## Validation evidence

The pre-change checkpoint-shell baseline passed:

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2638 filtered out; finished in 0.04s
```

The post-change focused suite passed:

```text
test result: ok. 88 passed; 0 failed; 3 ignored; 0 measured; 2555 filtered out; finished in 1.79s
```

The expanded Nix integration check passed:

```text
test result: ok. 88 passed; 0 failed; 3 ignored; 0 measured; 2555 filtered out; finished in 0.43s
```

The four new cases cover successful validation, rejected validation, failed cleanup, and changed copy content. Both successful and rejected paths preserve the pre-existing native payload.

Additional checks passed:

- root-package formatting;
- strict first-party Clippy with `--no-deps` and `-D warnings`;
- the architecture checker with zero findings and nine negative fixtures;
- machine contracts with 34 contracted and 68 classified surfaces;
- the pinned Tiger Style Nix check;
- the Nix architecture check;
- Cairn validation and all three change gates;
- Tracey coverage at 157/157;
- staged and unstaged whitespace checks.

Commands, outputs, and exit statuses are in this directory. Relevant pueue tasks were `1653`, `1674`, `253`, `254`, `256`, `265`, `266`, and `273`.

## Preserved failed check

`quality.attempt1.log` records a stale producer identity for `bootstrap.dev-resume-report`. The repository generator refreshed that identity. `machine-refresh.log` records the exact one-field inventory change and the successful rerun.

The schema, contract, fixtures, consumer policy, and compatibility freshness values did not change.

## Remaining limits

This repair does not implement publication at each completed stage. I3 remains open. The runtime audit found that manifest publication waits for full cold-run success.

The remote cold run still uses immutable source `97f47ae2a644651734324f44991cd4cae847499a`. This repair does not change that binary, source profile, or running process.

V2, V3, and V4 remain open. No archive, publication, promoted-proof, compiler-correctness, or general filesystem race-safety claim follows from these tests.
