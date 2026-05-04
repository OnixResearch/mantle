# Make 3.82 validation after TinyCC warning-format repair (r1)

Date: 2026-05-04

## Command

`crunch bootstrap validate bootstrap/make-tcc.ncl --warmup bootstrap/tinycc.ncl --resume` using the store/state from `diag-tcc27-warning-format-fixed-r2`, so the repaired TinyCC output was reused.

## Result

- Status: `build-failed`
- Build exit code: `1`
- Warmup `bootstrap/tinycc.ncl`: `passed` exit `0`
- Derivation log: `make-tcc-warning-format-fixed-r1.derivation.log`

Progress: TinyCC diagnostics are now expanded to concrete filenames/lines/symbols throughout the Make compile transcript; the previous literal `%s:%d` placeholder blocker is gone (`placeholder_present=False`).

Remaining blocker: `bootstrap/make-tcc.ncl` still fails at the `./make --version` verification step with builder exit 60 after printing:

```text
GNU Make 3.82
Built for unknown
```

V3 remains incomplete because there is still no accepted Make output path and no simple-Makefile smoke. The next repair should focus on why the produced Make exits 60 after version output, not on TinyCC diagnostic formatting.

Evidence files:

- `make-tcc-warning-format-fixed-r1-validation-summary.json`
- `make-tcc-warning-format-fixed-r1-validation-summary.md`
- `make-tcc-warning-format-fixed-r1-build.stdout.log`
- `make-tcc-warning-format-fixed-r1-build.stderr.log`
- `make-tcc-warning-format-fixed-r1-warmup-tinycc.stdout.log`
- `make-tcc-warning-format-fixed-r1-warmup-tinycc.stderr.log`
- `make-tcc-warning-format-fixed-r1-doctor.json`
- `make-tcc-warning-format-fixed-r1.derivation.log`
