# I2-I3 produce-aterm evidence

Task-ID: I2, I3
Covers: r[foreign_derivation_import.prefix_aware_aterm]
Date: 2026-08-01

## Question

Can the Mantle CLI emit canonical foreign import artifacts from explicit Nix and Guix ATerm bundles without frontend commands or partial outputs?

## Inspected evidence

- `src/foreign_import_cmd.rs`
- `src/foreign_derivation_import.rs`
- `tests/foreign_import_cli.rs`
- `tests/fixtures/foreign-import/guixpkgs-hello-root.drv`
- `tests/fixtures/foreign-import/guixpkgs-hello-source.drv`
- Pueue task `7854`: `nix develop -c cargo test -p mantle --test foreign_import_cli`
- Pueue task `7855`: `nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import`
- Pueue task `7848`: focused tests plus first-party Clippy

## Decision

I2 and I3 are complete. `produce-aterm` accepts explicit `--drv` mappings or one `--drv-dir`. The shell performs bounded reads and delegates parsing and lowering to pure core functions.

The Nix fixture produces the same graph and package index as `produce-nix`. The Guix fixture preserves `/gnu/store`, fixed-output data, input edges, builder data, arguments, and environment data.

The CLI tests use a fake `PATH`. They cover malformed ATerm, mixed prefixes, missing inputs, duplicate logical paths, non-UTF-8 bytes, oversized files, and input-mode conflicts. Rejected inputs emit no graph or index.

Exact results:

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1991 filtered out; finished in 0.01s
```

The focused first-party Clippy command completed successfully. Vendored `snix-castore` emitted one existing `dead_code` warning.

## Owner

Mantle foreign derivation import.

## Next action

Implement I4 through I7: deterministic dependency ordering, exact path maps, resolved registration, and negative graph compiler tests.
