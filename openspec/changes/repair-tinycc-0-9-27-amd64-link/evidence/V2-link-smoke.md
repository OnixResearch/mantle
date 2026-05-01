# V2 Compile/link/execute smoke

Task-ID: V2
Covers: bootstrap.compiler.tinycc.0.9.27.amd64.static-link

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/tcc-got-static4-store" \
  --state-dir "$PWD/.crunch-drain/tcc-got-static4-state" \
  bootstrap/diag-tcc-link-smoke.ncl
```

## Result

Status: PASS
Crunch-reported exit status: 0
Wrapper exit status: 1 due interactive shell logout hook (`__ETC_BASHLOGOUT_SOURCED: unbound variable`) after the command had already printed `exit status: 0`.
Transcript: `evidence/V2-static-got-repair-link-smoke.log`
Output path: `.crunch-drain/tcc-got-static4-store/p89ps590vql5g4hghmv97k6yvl0ckq0h-diag-tcc-link-smoke`

The repaired TinyCC output now compiles, statically links, chmods, and executes `int main(void) { return 0; }` inside the Crunch diagnostic derivation. A host follow-up also ran the exported binary successfully:

```sh
.crunch-drain/tcc-got-static4-store/p89ps590vql5g4hghmv97k6yvl0ckq0h-diag-tcc-link-smoke/bin/hello
# exit 0
```

Host `readelf -S` confirms the executable still contains `.plt` and `.got`; inspecting the `.got` bytes now shows non-zero populated entries instead of the all-zero GOT observed in the failing runtime evidence.
