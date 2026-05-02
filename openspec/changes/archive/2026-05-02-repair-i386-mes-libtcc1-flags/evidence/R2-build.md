# R2 build evidence

Task-ID: R2
Covers: bootstrap.i386-mes-libtcc1-flags.evidence

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute --store "$PWD/.crunch-drain/make-diag-stage2-store" --state-dir "$PWD/.crunch-drain/make-diag-stage2-state" bootstrap/spike-i386-mes-runtime-layout.ncl
```

Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/make-diag-stage2-store/79wkxsabqs8czm6h6y660bsnn6sjl5c2-spike-i386-mes-runtime-layout`

Summary:

```text
status=blocked
blocked_step=tcc27_compile_object
blocked_rc=139
note=The i386 Mes runtime/header layout proof found the first concrete blocker before tcc27 can use the layout.
```

Key return codes:

- `runtime_libtcc1_object`: `0`
- `runtime_libtcc1_archive`: `0`
