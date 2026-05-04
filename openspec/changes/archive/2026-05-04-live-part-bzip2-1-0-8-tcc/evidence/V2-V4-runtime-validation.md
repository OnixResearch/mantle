# V2-V4 runtime validation

## Command

```sh
mkdir -p .crunch-drain/bzip2-tcc-store .crunch-drain/bzip2-tcc-state \
  openspec/changes/archive/2026-05-04-live-part-bzip2-1-0-8-tcc/evidence
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ./target/debug/crunch bootstrap validate \
    --store "$PWD/.crunch-drain/bzip2-tcc-store" \
    --state-dir "$PWD/.crunch-drain/bzip2-tcc-state" \
    --resume \
    --evidence-dir "$PWD/openspec/changes/archive/2026-05-04-live-part-bzip2-1-0-8-tcc/evidence" \
    bootstrap/bzip2-tcc.ncl
```

## Result

- Status: `Passed`
- Build exit code: `0`
- Output path: `.crunch-drain/bzip2-tcc-store/an6izlkmrl9392q2617arqgck4bk9i4x-bzip2-1.0.8-tcc`
- Logical path: `/crunch/store/an6izlkmrl9392q2617arqgck4bk9i4x-bzip2-1.0.8-tcc`
- Build report: `evidence/build.stdout.log`
- Summary: `evidence/validation-summary.{json,md}`
- Derivation log copied to `evidence/bzip2-tcc.drv.log`.

## Smoke

The validation derivation checked the installed output contract:

- `bin/bzip2 --help`
- `bin/bunzip2 --help`
- `bin/bzcat --help`
- `bin/bzip2recover`

A host-side smoke against the produced output also round-tripped a fixture:

```sh
printf 'crunch-bzip2-smoke\n' > plain.txt
$out/bin/bzip2 -k plain.txt
$out/bin/bunzip2 -c plain.txt.bz2 > roundtrip.txt
cmp plain.txt roundtrip.txt
$out/bin/bzcat plain.txt.bz2 | grep '^crunch-bzip2-smoke$'
```

## Leakage scan

`validation-summary.json` reports `leakage_findings: []`; `build.stdout.log` reports `hermeticity_audit_events: []` and no diagnostic persistence failures.
