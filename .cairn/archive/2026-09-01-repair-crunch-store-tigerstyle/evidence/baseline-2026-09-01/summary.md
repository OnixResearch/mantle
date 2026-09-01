# `crunch-store` Tiger Style baseline

## Verdict

The strict baseline fails before source changes. No allowance or baseline is
configured.

## Counts

Focused command findings by file:

- `provenance.rs`: 60;
- `roots.rs`: 26;
- `nario.rs`: 21;
- `gc.rs`: 18;
- `overlay.rs`: 15;
- `pull.rs`: 12;
- `handle.rs`: 10;
- `http_closure.rs`: 8;
- `composition.rs`: 6;
- `layer.rs`: 3;
- `retention.rs`: 2;
- `query.rs`: 1;
- `capability.rs`: 1.

Total focused findings: 183 across 13 files.

The repository Tiger derivation records 139 location-backed findings across the
same 13 files before it stops.

## Test baseline

`nix develop -c cargo test -p crunch-store --lib --tests` passes:

- 357 unit tests;
- 2 integration tests;
- zero failures.

## Non-claims

This baseline does not prove store correctness, Tiger conformance, full-check
success, or release eligibility.
