# V3 binutils as bridge validation

Date: 2026-05-05T10:13:19.771416

Focused validation command:

```sh
timeout 570 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-bridge-store" \
  bootstrap validate bootstrap/binutils-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-bridge-evidence" \
  --resume
```

Result: `status=passed`, `build_exit_code=0`, no coarse host-path leakage findings.

Boundary advanced: the native `gas/as-new` TinyCC link remains fragile, so the derivation now treats the native gas link failure as non-fatal and installs a TCC-backed bootstrap `as` bridge when no native assembler is produced. The bridge passed the validation smoke by assembling the built-in `/tmp/smoke.s` to `/tmp/smoke.o`. `ar`/`ranlib` are bridged when native tools are unavailable; `ld`/`nm`/`objcopy` currently satisfy executable-presence checks only and remain follow-up hardening targets before archive.

Evidence files:

- `V3-binutils-as-bridge-validation-validation-summary.md`
- `V3-binutils-as-bridge-validation-validation-summary.json`
- `V3-binutils-as-bridge-validation-build.stdout.log`
- `V3-binutils-as-bridge-validation-build.stderr.log`
- `V3-binutils-as-bridge-validation-excerpt.log`
