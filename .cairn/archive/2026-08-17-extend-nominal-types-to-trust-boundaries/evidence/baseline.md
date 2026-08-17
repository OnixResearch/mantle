# Nominal trust-boundary baseline

## Source

- Base revision: `99add13ed9574edc6889bef49f15bbf938a7fd70`
- StageX lineage fixture BLAKE3: `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- StageX receipt fixture BLAKE3: `7d07ca06898807a29422f5da0a0ad9827f780f3568f31984151f568bb48bb003`

At the base revision, `crunch-bootstrap-core` has 13 `Blake3Hex::new` call sites. The constructor is infallible and direct deserialization bypasses format admission.

`src/remote_build.rs` has 15 raw `request_id: String` fields and 16 raw `endpoint_id: String` fields. Credential-owned ticket, verifier, validity, use, build-time, and upload-limit types are already present.

`content_bound_requirements.rs` has five selected raw validation helpers for repository, requirement, specification-path, BLAKE3, and release values.

`fetcher.rs` has seven `ambiguous_params` allowances. `ca_plan.rs` has one.

## Tests

The baseline commands passed before core edits:

- `cargo test -p crunch-bootstrap-core`: 79 passed.
- `cargo test -p crunch-build`: 674 library tests, one export test, one positive doctest, and four compile-fail doctests passed.
- `cargo test -p crunch-pipeline`: 37 library tests and 19 integration tests passed; four probes were ignored.
- `cargo test -p crunch-release-core`: 233 passed.
- `cargo test -p mantle --bin mantle frontend_artifact_spec::`: 10 passed.
- `cargo test -p mantle --bin mantle remote_build::`: 144 passed.

The baseline proves only the observed source shape and test outcomes at the named revision.
