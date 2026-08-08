# I1 baseline evidence

Date: 2026-08-04. The archived producer implementation recorded the baseline before this change.

Commands:

```text
cargo test -p mantle --bin mantle foreign_derivation_import
cargo test -p mantle --bin mantle source_bundle
cargo test -p mantle --test foreign_import_cli
```

Results:

```text
foreign_derivation_import: test result: ok. 20 passed; 0 failed
source_bundle:             test result: ok. 85 passed; 0 failed
foreign_import_cli:        test result: ok. 14 passed; 0 failed
```

The CLI tests require `bwrap` on `PATH`. Missing `bwrap` is an environment failure, not a producer-code failure.
