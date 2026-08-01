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
