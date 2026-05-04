# V2 sed-tcc build validation

- Task-ID: V2
- Covers: `bootstrap.part.sed.4.0.9.tcc`
- Status: passed
- Command: `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store "$PWD/.crunch-drain/sed-tcc-store" --state-dir "$PWD/.crunch-drain/sed-tcc-state" bootstrap validate bootstrap/sed-tcc.ncl --evidence-dir openspec/changes/live-part-sed-4-0-9-tcc/evidence --resume`
- Build exit code: `0`
- Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/sed-tcc-store/2xngwf3d5m7gl54cph0s0x24vb5ykyps-sed-4.0.9-tcc`
- Logical output path: `/crunch/store/2xngwf3d5m7gl54cph0s0x24vb5ykyps-sed-4.0.9-tcc`
- Derivation log: `evidence/V2-sed-tcc-root-derivation.log`
- Runner summary: `evidence/validation-summary.md` / `evidence/validation-summary.json`
