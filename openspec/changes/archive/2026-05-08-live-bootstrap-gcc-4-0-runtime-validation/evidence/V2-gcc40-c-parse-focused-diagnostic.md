# GCC 4.0 c-parse focused diagnostic

Task-ID: V2 follow-up diagnostic
Covers: bootstrap.gcc40.runtime-validation

## Scope

Added `bootstrap/diag-gcc40-c-parse-boundary.ncl`, derived from `bootstrap/gcc-4.0.ncl`, to stop intentionally at the first non-generator GCC frontend boundary instead of continuing into install/smoke logic. This is a diagnostic target, not a production bootstrap part.

## Commands

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/diag-gcc40-c-parse-boundary.ncl
mkdir -p .crunch-drain/gcc40-cparse-diag-store .crunch-drain/gcc40-cparse-diag-evidence
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cparse-diag-store" \
  bootstrap validate bootstrap/diag-gcc40-c-parse-boundary.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-cparse-diag-evidence" \
  --resume
```

## Results

Source-pin audit:

```text
source-pin audit: 1 files, 1 fetch blocks, 0 issues
```

Focused validation:

- status: `build-failed`
- build_exit_code: `1`
- failure_class: `build`
- leakage_findings: only `/bin/` sandbox path matches (`count=31`)
- saved drv log: `evidence/V2-gcc40-c-parse-focused-diagnostic-drv.log`

Current focused boundary:

```text
gcc40-cc ... /tmp/gcc-build/gcc-4.0.4/gcc/c-parse.c -o c-parse.o
make: *** [c-parse.o] Segmentation fault (core dumped)
```

This confirms the c-parse failure is reproducible in a target whose purpose is only to reach that boundary. It does not prove a GCC output exists, so V4 remains open.

## Artifacts

- `evidence/V2-gcc40-c-parse-focused-diagnostic-build.stdout.log`
- `evidence/V2-gcc40-c-parse-focused-diagnostic-build.stderr.log`
- `evidence/V2-gcc40-c-parse-focused-diagnostic-validation-summary.json`
- `evidence/V2-gcc40-c-parse-focused-diagnostic-validation-summary.md`
- `evidence/V2-gcc40-c-parse-focused-diagnostic-doctor.json`
- `evidence/V2-gcc40-c-parse-focused-diagnostic-drv.log`
