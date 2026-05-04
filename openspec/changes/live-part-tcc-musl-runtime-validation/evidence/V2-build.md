# V2 build evidence: tcc musl

Task-ID: V2
Covers: r[bootstrap.part.tcc.musl.runtime-validation]
Status: captured

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/tcc-musl-default-store" \
  bootstrap validate bootstrap/tcc-musl.ncl \
  --evidence-dir "$PWD/.crunch-drain/tcc-musl-default-evidence" \
  --resume
```

## Result

Validation now fails at the first musl prerequisite, not at `tcc-0.9.27-musl-prep`:

- failed root: `tcc-0.9.27-musl`
- derivation: `/crunch/store/2azrpcqivj29rfn96avl1bgxsivcic48-tcc-0.9.27-musl.drv`
- phase: `build`
- error class: `builder`
- message: `dependency musl-1.1.24-tcc.drv failed`

This records the new downstream boundary after the prep compiler repair.

Direct `musl-1.1.24-tcc` evidence: see `evidence/V2-musl-1.1.24-tcc-direct-validation.md` and root derivation log `evidence/V2-musl-1.1.24-tcc-direct-root-derivation.log`. The exploratory repair advanced through configure/startup-object barriers but still failed at musl object compilation, so no implementation patch is committed.
