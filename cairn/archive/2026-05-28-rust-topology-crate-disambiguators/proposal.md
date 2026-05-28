# rust-topology-crate-disambiguators

## Why

Native topology now reaches crates that require multiple versions of the same crate name in one rustc session. `object_store@0.13.2` directly uses `rand@0.10.1`, while its dependency closure also brings crates such as `ring` that use older same-name dependencies like `getrandom@0.2.x`.

Mantle currently invokes rustc without Cargo-style crate disambiguators. The produced rlibs use bare names like `libgetrandom.rlib`, and rustc can reject direct dependencies with `E0463: can't find crate` when same-name versions are present in dependency search paths.

## Change

Add deterministic `-C metadata=<hash>` to every native rustc unit derivation. The hash is derived from reviewable unit identity fields, not ambient target directories. This mirrors Cargo's crate-disambiguation role while preserving Mantle's explicit output paths and receipts.

## Success

- Focused tests prove same crate names from different package IDs receive different metadata disambiguators.
- Focused tests prove the same unit identity receives stable metadata across derivation generation.
- Clean self-probe advances beyond `object_store@0.13.2` `rand` loading or records the next deterministic frontier.
