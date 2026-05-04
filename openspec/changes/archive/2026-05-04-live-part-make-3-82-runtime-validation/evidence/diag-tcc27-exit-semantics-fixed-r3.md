# TinyCC/Mes exit semantics fixed

Task-ID: V3
Covers: bootstrap.part.make.3.82.runtime-validation

## Result

After patching the x86_64 Mes `_exit` inline assembly used by `bootstrap/tinycc.ncl`, focused TinyCC-built programs now preserve process termination status instead of returning syscall number 60 for `exit(0)`. This rerun includes in-derivation assertions for all expected return codes.

Validation command:

```sh
crunch --json --store .crunch-drain/diag-tcc27-exit-semantics-fixed-r2-store \
  --state-dir .crunch-drain/diag-tcc27-exit-semantics-fixed-r2-state \
  bootstrap validate bootstrap/diag-tcc27-exit-semantics.ncl \
  --warmup bootstrap/tinycc.ncl --resume \
  --evidence-dir target/bootstrap-validation/diag-tcc27-exit-semantics-fixed-r3
```

Validation summary: `status=passed`, `build_exit_code=0`, `leakage_findings=[]`.

Observed RC matrix from `/tmp/hermes-worktrees/crunch/make-exit60-diagnostic/.crunch-drain/diag-tcc27-exit-semantics-fixed-r2-store/z4c8hlryivx75854m1xp3my0iv5jmm93-diag-tcc27-exit-semantics/src`:

```text
exit0.rc=0
exit7.rc=7
return0.rc=0
return7.rc=7
underscore_exit0.rc=0
underscore_exit7.rc=7
```

Evidence files: `diag-tcc27-exit-semantics-fixed-r3-*.log`, `diag-tcc27-exit-semantics-fixed-r3-validation-summary.json`, and `diag-tcc27-exit-semantics-fixed-r3.derivation.log`.
