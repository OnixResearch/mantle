# Nickel evaluator cohort

Mantle binds embedded evaluation and command-line validation to one reviewed Nickel cohort.

| Surface | Identity |
|---|---|
| CLI | Nickel `1.17.0` |
| CLI source | `nickel-lang/nickel` commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426` |
| CLI Rust requirement | `1.89` |
| Embedded facade | `nickel-lang` `2.2.0` |
| Embedded core | `nickel-lang-core` `0.18.0` |
| Vendor process | `cargo vendor --locked --versioned-dirs vendor-deps` |

`flake.nix` selects the pinned upstream CLI package. It does not use the Nickel version from the ambient Nixpkgs revision.

The source of truth is `config/nickel-cohort.ncl`. Its generated JSON is copied into bootstrap and release-source evidence. `bootstrap/evidence/nickel-1.17-vendor-manifest.json` binds each locked Nickel package to deterministic BLAKE3 tree identity, file count, byte count, and preserved MIT license.

Run the local checks with:

```sh
nix run .#check-nickel-configs
nix build .#checks.x86_64-linux.nickel-cohort --no-link -L
nix develop -c cargo test -p crunch-eval --lib
```

Refresh an absent vendor tree through the repository-owned importer:

```sh
nix develop -c sh -c \
  'cargo -Zscript scripts/refresh-nickel-cohort.rs --root . --cargo "$(command -v cargo)" --execute'
nix develop -c cargo -Zscript scripts/check-nickel-cohort.rs --require-vendor .
```

The importer refuses to replace an existing `vendor-deps/` directory. A normal clone can run the tracked check without materializing that ignored directory.

The historical V98 fixed-point evidence remains bound to its original source. This cohort change does not claim a current fixed point, evaluator correctness, derivation correctness, compiler correctness, or release eligibility.
