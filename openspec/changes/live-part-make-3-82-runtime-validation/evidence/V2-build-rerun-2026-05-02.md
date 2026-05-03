# V2 build rerun (2026-05-02)

Task-ID: V2
Covers: bootstrap.part.make.3.82.runtime-validation

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/make-3-82-long-store" \
  --state-dir "$PWD/.crunch-drain/make-3-82-long-state" \
  bootstrap validate bootstrap/make-tcc.ncl \
  --evidence-dir openspec/changes/live-part-make-3-82-runtime-validation/evidence \
  --resume
```

## Result

- Runner exit: 1
- Runner status: `build-failed`
- Doctor/preflight: passed
- Build attempted: yes
- Build failure class: `build`
- Root: `make-3.82-tcc`
- Derivation: `/crunch/store/bwkphpkw52sz9zf3xqlyj8bqnqavn1ym-make-3.82-tcc.drv`
- Builder failure: exit status 139 (`Segmentation fault (core dumped)`)
- Runner leakage findings: none

## Evidence files

- `validation-summary.json`
- `validation-summary.md`
- `build.stdout.log`
- `build.stderr.log`
- `build.derivation-2026-05-02.log`

## Interpretation

The rerun rebuilt through the prerequisite chain and reproduced the Make 3.82 runtime blocker. The failure remains before a usable GNU Make output is produced: TinyCC/Mes emits literal diagnostic format strings such as `%s:%d` and the `make-3.82-tcc` builder exits 139.

V3 remains blocked because there is no produced `make` binary to smoke-test. V5 remains pending until either the Make runtime smoke passes or this runtime-validation change is explicitly superseded by a focused TinyCC/Mes runtime repair change.
