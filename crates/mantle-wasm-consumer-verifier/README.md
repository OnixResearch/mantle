# mantle-wasm-consumer-verifier

Published consumer contract for verifying Mantle WebAssembly component
materialization bundles (`mantle-wasm-component-materialization-bundle-v1`).

Independent consumers such as Kamacite, Aspen, Animus, and Lattice can pin
this crate to remeasure exact bundle members before runtime admission
without importing Mantle build, store, scheduler, release, or CLI authority.

## API

- `verify_consumer_bundle(MaterializationBundle) -> ConsumerVerificationReport`
  runs the structural layer over in-memory bundle data. Every structural and
  canonical-identity decision delegates to `crunch-wasm-component-core`; this
  facade never duplicates decision logic.
- `ConsumerRoot::open(dir)` opens one explicit capability root.
  `verify_bundle(bundle)` then remeasures every required member under that
  root: no absolute escapes, no parent traversal, no symlinks, regular files
  only, named byte bounds (`MAX_MEMBER_BYTES`, `MAX_TOTAL_REMEASURED_BYTES`),
  BLAKE3 remeasurement, declared-length comparison, and per-member identity
  comparison. Roles may share one logical path; each unique path is measured
  once and judged against every role's declared identity.
- The report (`mantle-wasm-consumer-verification-report-v1`) binds schemas,
  bundle and runtime-profile identities, member roles and BLAKE3 values,
  completed layers, ordered blockers, one status (`verified`, `blocked`,
  `rejected`), and fixed non-claims. It never carries logical store paths,
  payloads, credentials, environment values, or raw tool diagnostics.

Missing required bytes produce `blocked`. Drifted bytes produce the
`consumer-member-bytes-drift` blocker and `rejected`.

## Source acquisition

Pin an immutable revision of this repository and import only
`crates/mantle-wasm-consumer-verifier` plus its one path dependency,
`crates/crunch-wasm-component-core`. Both are `no_std + alloc` at the core
edge; the file shell requires the default `std` feature on a Linux host.
Consumers must not depend on sibling worktree paths.

## Consumer flow

1. Receive a bundle document from the producer.
2. Parse it as `MaterializationBundle`.
3. Run `verify_consumer_bundle` for the structural verdict.
4. Open the consumer-owned member root and run
   `ConsumerRoot::verify_bundle` for byte remeasurement.
5. Consume the bounded report; never treat `blocked` or `rejected` output as
   verified, and never extend the report's non-claims.

## Claim boundary

A passing report does not prove source trust, compiler correctness,
component behavior, runtime isolation, authority, reproducibility,
deployment safety, or release eligibility. The report's `non_claims` field
is fixed by `CONSUMER_VERIFIER_NON_CLAIMS` and cannot be extended by
callers.
