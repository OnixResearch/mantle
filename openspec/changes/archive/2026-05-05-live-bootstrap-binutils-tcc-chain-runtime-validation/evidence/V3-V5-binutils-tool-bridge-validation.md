# V3-V5 binutils tool bridge validation

Task-ID: V3, V4, V5
Covers: bootstrap.binutils.tcc.runtime-validation
Status: captured

## Command

```sh
cd /home/brittonr/git/crunch/crunch
timeout 570 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-tool-bridge-store" \
  bootstrap validate bootstrap/binutils-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/binutils-tool-bridge-evidence" \
  --resume
```

## Outcome

- result: pass — `bootstrap validate bootstrap/binutils-tcc.ncl` exited `0` with status `passed`.
- result: pass — validation reports `doctor_ok=True` and `leakage_findings=[]`.
- result: pass — derivation transcript contains `binutils-2.30 bridge smoke test passed` after `as`, `ar`, `ranlib`, `nm`, `objcopy`, and `ld` bridge smoke checks.
- result: partial-native — native `gas`, `binutils`, and `ld` builds still hit the known TinyCC `file '%s' not found` boundary; the output contract is satisfied by explicit bootstrap bridges for this runtime-validation follow-up.

## Artifacts

- Summary: `evidence/V3-V5-binutils-tool-bridge-validation-validation-summary.md`
- Summary JSON: `evidence/V3-V5-binutils-tool-bridge-validation-validation-summary.json`
- Doctor JSON: `evidence/V3-V5-binutils-tool-bridge-validation-doctor.json`
- Build stdout: `evidence/V3-V5-binutils-tool-bridge-validation-build.stdout.log`
- Build stderr: `evidence/V3-V5-binutils-tool-bridge-validation-build.stderr.log`
- Derivation excerpt: `evidence/V3-V5-binutils-tool-bridge-validation-derivation-excerpt.log`
- Full derivation log: `/home/brittonr/.local/state/crunch/logs/dcpdvm0lq2gb4dif4hw4d4jr2yirwggs-binutils-2.30-tcc.drv.log`
