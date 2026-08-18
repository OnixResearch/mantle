# I1-I4 execution-profile admission

Task-ID: I1, I2, I3, I4
Covers: r[foreign_derivation_import.realization_adapter], r[foreign_derivation_import.execution_profile]
Date: 2026-08-02

## Result

Mantle now validates executable plans, import-receipt links, selected roots,
profile sets, native units, profile bindings, and local-only routing before
store mutation.

Each foreign derivation carries the reserved
`__MANTLE_FOREIGN_EXECUTION_PROFILE_V1` field. Its canonical profile BLAKE3 is
present before HDM, output-path, derivation-path, and ATerm computation. The
ordinary registry stores the verified profile, and build-request creation
requires it explicitly.

Typed Nickel contracts and deterministic JSON exports define the Nix and Guix
profiles. The Guix profile does not provide `/bin/sh`.

## Baseline

Pueue task `7898` established these baseline results:

```text
crunch-build build_request: 37 passed
crunch-build fetch_build_service: 20 passed
foreign_import_cli: 12 passed
```

The listed `check-nickel-configs` command was absent from the baseline shell.
This phase added the command and its deterministic positive and negative checks.

## Verification

Pueue tasks `7904`, `7914`, and `7916` verified:

```text
execution_profile: 7 passed
build_request: 40 passed
registry: 17 passed
foreign_realization: 3 passed
foreign_graph_compiler: 8 passed
foreign_executable_plan: 3 passed
foreign_import_cli: 12 passed
check-nickel-configs: passed
focused crunch-build Clippy: passed
focused mantle Clippy: passed
```

The negative coverage rejects unknown fields, invalid limits, reserved-field
collisions, stale digests, forbidden `/bin/sh`, undeclared network authority,
setid, syscall exceptions, writable-path escapes, protected environment
overrides, missing profiles, stale receipts, unknown roots, and remote routes.

Only the existing vendored `snix-castore` dead-code warning appeared.
