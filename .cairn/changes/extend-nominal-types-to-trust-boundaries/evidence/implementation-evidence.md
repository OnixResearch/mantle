# Nominal trust-boundary implementation evidence

## Result

Mantle now admits role-specific values before selected bootstrap, StageX, remote, content-bound, frontend, fetch, and content-addressed planning decisions use them.

Structural wire DTOs remain separate. Compatibility adapters preserve accepted field names and scalar spellings.

## Implemented boundaries

- `crunch-bootstrap-core`: fallible custom-Serde `Blake3Hex`, StageX role digests, stage/artifact/executable IDs, absolute executable paths, unit limits, and resolved lineage node roles.
- Remote protocol: request, session, endpoint, and output IDs. Credential-owned ticket and unit types remain separate.
- `crunch-release-core`: repository, requirement, release, specification, evidence-path, and checked digest aggregates.
- Frontend artifacts: spec, validator, artifact, target, build-root, and spec-hash roles before attestation projection.
- `crunch-build`: typed fetch requests, URL/revision/archive/output roles, CA derivation/output requests, logical prefixes, derivation keys, and provisional/final path roles.
- Cairn and Octet policy: ten sorted Mantle nominal domains with fallible-constructor and compatibility scopes.

## Baseline

See `baseline.md`. The baseline was revision `99add13ed9574edc6889bef49f15bbf938a7fd70`.

## Positive and negative validation

- `crunch-bootstrap-core`: 86 tests and two compile-fail doctests passed.
- `crunch-build`: 677 library tests, export test, positive doctest, and five compile-fail doctests passed.
- `crunch-pipeline`: 37 library and 19 integration tests passed; four probes remained ignored.
- `crunch-release-core`: 235 tests and one compile-fail doctest passed.
- Frontend artifact tests: 11 passed.
- Remote build tests: 146 passed.
- StageX source-bundle compatibility tests: 87 passed.
- No-std Wasm checks passed for `crunch-bootstrap-core` and `crunch-release-core`.
- Focused Clippy passed for all four changed first-party crates with warnings denied and dependencies excluded.
- The generated policy is fresh and both Nickel sources typecheck.
- Cairn validation and proposal, design, and tasks gates passed.
- Traceability coverage passed: 155 of 155 referenced.

Negative tests cover empty, oversized, control-bearing, malformed, wrong-role, missing-reference, unsafe-path, zero, over-limit, bad-Serde, wrong-prefix, malformed-revision, endpoint-mismatch, unsupported-validator, and digest-domain cases.

## Bounded blockers

`check-first-party-tigerstyle.sh` reached inherited findings in unchanged `crunch-gc-core` and `crunch-overlay-core`. It reported no finding in a changed nominal file. No baseline, warning budget, or disabled lint was added.

`nix flake check --no-build -L` reached the pre-existing SpaceWasm package and failed because its upstream source is unavailable and a cached derivation path is invalid.

Broad dependency Clippy also reports existing vendored `fuse-backend-rs` findings. Focused changed-crate Clippy passes.

## Non-claims

The nominal types prove local syntax, bounds, role separation, selected relationships, and wire projection only.

They do not prove filesystem authority, artifact correctness, source trust, remote peer trust, sandbox enforcement, compiler correctness, evidence truth, or release eligibility.
