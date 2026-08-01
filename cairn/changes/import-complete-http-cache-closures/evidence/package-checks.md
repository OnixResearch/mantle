# Package checks

Date: 2026-08-01

Pueue task `7152` ran this exact sequence:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo check -p crunch-store -p mantle
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt --check -p crunch-store -p mantle -v
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo clippy -p crunch-store --lib --no-deps -- -D warnings
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo clippy -p mantle --bin mantle --test integration --no-deps -- -D warnings
```

Task `7152` completed successfully. Cargo reported only the existing vendored `snix-castore` dead-code warning for `directoryservice::combinators::Error::Unimplemented`. First-party Clippy accepted the changed library, binary, and integration test targets with `-D warnings`.

## Post-rebase regression

Main advanced with the independent StageX commit `93f8f4cb`. After rebasing, pueue task `7177` reran the full 53-test pull filter, the 10-test store-pull CLI filter, and both first-party Clippy commands. Task `7177` completed successfully with only the same vendored warning.
