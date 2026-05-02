# R3 runtime proof

Task-ID: R3
Covers: bootstrap.i386-tinycc26-object-emission.runtime-proof

## Commands

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/make-diag-stage2-store" \
  --state-dir "$PWD/.crunch-drain/make-diag-stage2-state" \
  bootstrap/diag-i386-tinycc26-emission.ncl

nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/make-diag-stage2-store" \
  --state-dir "$PWD/.crunch-drain/make-diag-stage2-state" \
  bootstrap/spike-i386-tinycc26-cross-smoke.ncl
```

## Diagnostic summary

Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/make-diag-stage2-store/rpg71pb1v9544gb465m9scxqf4dwk5l2-diag-i386-tinycc26-emission`

```text
diag-i386-tcc26 summary
assemble_object rc=0
compile_c_object rc=0
link_from_asm rc=0
link_from_object rc=0
run_i386-exit42-from-asm rc=42
run_i386-exit42-from-object rc=42
version rc=0
```

## Spike proof

Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/make-diag-stage2-store/l3yqzskidhag8wwhhyvrxv4fjp1xdf3y-spike-i386-tinycc26-cross-smoke`

```text
TinyCC 0.9.26 i386 target produced native i386 executable with rc=42
```

The final spike output contains:

- `bin/tcc26-i386`: x86_64-hosted static TinyCC 0.9.26 targeting i386.
- `bin/i386-exit42`: 32-bit Intel 80386 static ELF.

The builder executed `./i386-exit42`, observed exit code `42`, and required `test "$rc" = 42`.
