# Baseline verification

Before the core change, these commands passed on commit `01223ed6`:

```text
nix develop -c cargo test -p mantle --bin mantle build_correctness
nix develop -c cargo test -p mantle --test foreign_import_cli
```

The foreign CLI suite reported 14 passing tests and no failures. This baseline
established that the existing realization adapter and build-correctness checks
were green before provenance scanning changed them.
