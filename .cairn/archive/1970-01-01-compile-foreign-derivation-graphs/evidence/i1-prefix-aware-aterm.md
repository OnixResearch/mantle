# I1 prefix-aware ATerm evidence

Task-ID: I1
Covers: r[foreign_derivation_import.prefix_aware_aterm]
Date: 2026-08-01

## Question

Does the pure Mantle core parse bounded `/nix/store` and `/gnu/store` ATerm bundles and reject each required malformed input class?

## Inspected evidence

- `src/foreign_derivation_import.rs`
- Pueue task `7820`: `nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import`
- Pueue task `7821`: `nix develop -c cargo clippy -p mantle --bin mantle --no-deps -- -D warnings`

## Decision

I1 is complete. The parser preserves the declared prefix and fixed-output facts. It rejects malformed ATerm, mixed prefixes, malformed store paths, missing inputs, duplicate logical paths, non-UTF-8 input, and limit overruns.

The focused test result was:

```text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1991 filtered out; finished in 0.03s
```

The focused first-party Clippy command completed successfully. Vendored `snix-castore` emitted one existing `dead_code` warning.

## Owner

Mantle foreign derivation import.

## Next action

Implement I2 as a thin `foreign-import produce-aterm` filesystem shell over the pure parser.
