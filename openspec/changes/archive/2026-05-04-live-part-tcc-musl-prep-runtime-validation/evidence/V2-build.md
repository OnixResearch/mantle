# V2 build evidence: tcc musl prep

Task-ID: V2
Covers: r[bootstrap.part.tcc.musl.prep.runtime-validation]
Status: captured

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/tcc-musl-prep-default-store" \
  bootstrap validate bootstrap/tcc-musl-prep.ncl \
  --evidence-dir "$PWD/.crunch-drain/tcc-musl-prep-default-evidence" \
  --resume
```

## Result

- validation status: `passed`
- build exit code: `0`
- derivation: `/crunch/store/psj38qn7b5yswggyv53fv552icz2i116-tcc-0.9.27-musl-prep.drv`
- logical output: `/crunch/store/bjzmbz7dn1l7nr168aanw9mj1piw2rbi-tcc-0.9.27-musl-prep`
- exported output: `/home/brittonr/git/crunch/crunch/.crunch-drain/tcc-musl-prep-default-store/bjzmbz7dn1l7nr168aanw9mj1piw2rbi-tcc-0.9.27-musl-prep`
- artifact attestation: `/home/brittonr/.local/state/crunch/attestations/artifacts/c044139a47767b494f0af7655412293b81169865d00f65ea26ec125b150138d0.json`

The previous `elf.h:51` / `Elf32_Xword` blocker is resolved. The derivation now compiles `tcc.c` to `tcc-musl-prep.o`, links `tcc-musl-prep`, and exports the bridge compiler output.

## Transcript

- Build report: `evidence/build.stdout.log`
- Root derivation log: `evidence/V2-tcc-0.9.27-musl-prep-root-derivation.log`
