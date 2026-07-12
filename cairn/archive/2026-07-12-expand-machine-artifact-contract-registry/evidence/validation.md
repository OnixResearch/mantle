# Machine artifact contract registry validation

Captured: 2026-07-12

Task-ID: Phase 1 Phase 2 Phase 3 Phase 4 Phase 5
Covers: mantle.machine_artifact_contracts.inventory, mantle.machine_artifact_contracts.authority, mantle.machine_artifact_contracts.registry_rail, mantle.machine_artifact_contracts.contract_vocabulary, mantle.machine_artifact_contracts.freshness, mantle.machine_artifact_contracts.initial_cohort, mantle.machine_artifact_contracts.fixtures, mantle.machine_artifact_contracts.versioning, mantle.machine_artifact_contracts.runtime_boundary

## Result summary

- 44 public machine-artifact families are classified.
- 15 surfaces are contracted: the 13 original cohort surfaces plus the stable OCI export/import reports.
- Rust DTOs remain runtime authority; generated Nickel contracts run only in checker/integration tests.
- No prior schema versions are declared supported. The checker rejects any prior version unless both an explicit converter and migration fixtures are registered.
- The accepted spec was synced and the completed change was archived after implementation and gate evidence; no push was run.

## Checker self-test, generation, and freshness

```text
$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
machine schema contract self-test: PASS

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --generate
machine schema contract generation: PASS (13 contracted, 42 classified)

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (13 contracted, 42 classified)
```

The self-test includes positive validation and negative probes for unsupported schema keywords, permissive `Dyn` generation, named-bound rendering, exact UTF-8 byte limits, serde rename/skip/requiredness parity, invalid digests/references/bounds/unknown fields/cross-field relations, duplicate registry ownership, missing/unclassified producer markers, unclassified root JSON serializers, runtime references to review-only Nickel contracts, undeclared compatibility versions, and stale BLAKE3 inputs.

## Embedded Nickel fixture validation

```text
$ nix develop -c cargo test -q -p mantle --test machine_schema_contracts -- --nocapture
running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s
```

This evaluates the typed Nickel inventory and checks every contracted positive and categorized negative fixture against its generated contract.

## Rust producer fixture parity

```text
$ nix develop -c cargo test -q -p mantle --bin mantle machine_contract_producer_tests:: -- --nocapture
running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1285 filtered out; finished in 0.00s

$ nix develop -c cargo test -q -p mantle --bin mantle release_envelope_serializes_to_registered_positive_fixtures -- --nocapture
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1289 filtered out; finished in 0.00s
```

## Consumer-owner focused tests

```text
$ nix develop -c cargo test -q -p mantle --bin mantle operator_diagnostics::tests -- --nocapture
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1278 filtered out; finished in 0.00s

$ nix develop -c cargo test -q -p mantle --bin mantle build_report::tests -- --nocapture
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1277 filtered out; finished in 0.00s

$ nix develop -c cargo test -q -p mantle --bin mantle realization_routing::tests -- --nocapture
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1269 filtered out; finished in 0.00s

$ nix develop -c cargo test -q -p mantle --bin mantle portable_receipt::tests -- --nocapture
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1259 filtered out; finished in 0.03s

$ nix develop -c cargo test -q -p mantle --bin mantle source_bundle::tests -- --nocapture
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 1250 filtered out; finished in 0.00s

$ nix develop -c cargo test -q -p mantle --bin mantle nickel_export::tests -- --nocapture
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1287 filtered out; finished in 0.00s
```

## Scoped formatting

```text
$ nix develop -c rustfmt --check --edition 2024 src/attest_cmd.rs src/machine_contract_producer_tests.rs src/structured_refactor.rs src/transcript_cmd.rs scripts/check-machine-schema-contracts.rs scripts/machine_schema_contracts/mod.rs scripts/machine_schema_contracts/model.rs scripts/machine_schema_contracts/registry.rs scripts/machine_schema_contracts/schema.rs scripts/machine_schema_contracts/instance.rs scripts/machine_schema_contracts/render.rs scripts/machine_schema_contracts/freshness.rs tests/machine_schema_contracts.rs tools/tracey_refs.rs
(exit 0)
```

`nix develop -c cargo fmt --check -p mantle -v` is not claimed as passing: it reports pre-existing unrelated formatting drift, including `tests/trust_policy_offline_rail.rs`. The scoped changed-file check above passes.

```text
$ git diff --check
(exit 0)
```

## Cairn validation and review gates

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 18,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 43,
  "valid": true
}
```

Initial pre-completion review receipts were PASS:

```text
proposal: receipt_hash=008b83cf7c24eec2e61cedc694f689da04deecce2b5f604c1afa445b8cbd1b3b verdict=PASS
 design: receipt_hash=415c65a2227a41d313ff738caf6c5e6967aa3fc519c714f8d8d8df67bc6e57aa verdict=PASS
  tasks: receipt_hash=f3b1c89d6b1787d128c7081f31e4ba4b9a5194fefbe58db58c82953ad841acc1 verdict=PASS
```

Final post-task-update validation and gate run:

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
valid=true changes=18 specs_validated=43 issues=[]

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal expand-machine-artifact-contract-registry --root .
input_hash=45cc9a0cfdfc45f3bc877359c5ae077f23ba3879affaf944030c5994edc7e4c1
receipt_hash=8ffb021f8dae691a0f12e82b7f72e0f128e5a1d47a47f0328c1175f78570da8a
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design expand-machine-artifact-contract-registry --root .
input_hash=29507ac028708bd22f00bf3715ab66d86200943397d8869374166eb0b24ef0d4
receipt_hash=ac5a2600fa563c5dd57bc0da78ff9f9d4c4ff4dfbda1f51ed4da5f90d89c6132
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks expand-machine-artifact-contract-registry --root .
input_hash=5092e48537e5e7b77051334dd1701bacd410892c0786a8f2226e7349fe8d5a2f
receipt_hash=7c08573032e431a025e8625923bec1314e813f8e379bcf828be488007cc757ca
issues=[] valid=true verdict=PASS
```

## Non-gating existing Tracey blocker

`cairn tracey coverage --root .` is not claimed as passing. The active `mantle-default` Tracey profile currently points only at the accepted `release-provenance` spec and its five evidence sources; it reports 55 pre-existing missing `mantle.release_provenance.*` references. It reports no dangling references and does not evaluate this active change's requirement source. Cairn validation and all three change gates pass independently.

## Adversarial integration review

The integration review found and closed generic false-pass paths that the initial
cohort did not exercise directly:

- nullable unions could contain multiple non-null types;
- `$ref` siblings and type-incompatible keywords could be accepted and then
  ignored by the generated predicate;
- collection bounds could be negative or fractional, and arrays could omit an
  item schema;
- nullable type handling could bypass a sibling `const` or `enum` constraint;
- missing invariant operands could be ignored by the Rust validator while the
  Nickel contract rejected them;
- the textual Rust-owner parser could be spoofed by comments or prefix-matching
  declarations and did not compare resolvable root scalar/container types;
- registry artifact paths were not normalized or confined to the contract
  subtree.

The repaired checker parses owner files with `syn`, rejects unsupported
serializer rewrites, validates root field names/requiredness/type categories,
uses exact nullable unions and root-definition-only references, validates
keyword/type compatibility and invariant declarations, keeps universal literal
constraints outside nullable type predicates, uses checked invariant sums, and
confines normalized artifact paths.

Current post-review evidence:

```text
$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
machine schema contract self-test: PASS

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (13 contracted, 42 classified)

$ nix develop -c cargo test -q -p mantle --test machine_schema_contracts -- --nocapture
running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.32s

$ nix develop -c cargo test -q -p mantle --bin mantle machine_contract -- --nocapture
running 6 tests
...
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1284 filtered out; finished in 0.00s

$ nix develop -c rustfmt --check --edition 2024 scripts/check-machine-schema-contracts.rs scripts/machine_schema_contracts/model.rs scripts/machine_schema_contracts/registry.rs scripts/machine_schema_contracts/schema.rs scripts/machine_schema_contracts/instance.rs scripts/machine_schema_contracts/render.rs
(exit 0)

$ git diff --check
(exit 0)
```

The attempted `cargo clippy -Zscript ...` invocation is not claimed as evidence:
this Cargo version dispatches that form to `cargo check` and rejects the script
path as an unexpected argument. The executable self-test/check, scoped rustfmt,
Rust producer fixtures, and Nickel integration tests are the validated rails.

## Post-integration DTO drift and OCI extension

Concurrent integration added two `BuildJsonReport` fields after the original
13-surface evidence was captured. The generic Rust-owner parity rail rejected
the stale schema with exact missing-field diagnostics for
`ast_grep_structural_evidence` and
`ast_grep_structural_evidence_diagnostics`. The build-report schema, fixtures,
generated contract, and producer freshness inputs now include both fields and
the bounded nested structural-evidence shape.

The accepted generic rail also exposed four previously unclassified OCI report
producer files. `OciExportReport` and `OciImportReport` are now registered as
contracted surfaces with Rust ownership, exact schemas, generated Nickel
contracts, Rust-generated golden fixtures, categorized negative fixtures,
version rejection, BLAKE3 freshness, SHA-256 role predicates, Mantle reference
predicates, and frontend-neutral non-claims.

Adversarial review found two integration-specific false-pass/failure paths and
closed them before completion:

- nullable schema fields were initially used for nested `Option` fields whose
  Rust serializers use `skip_serializing_if = "Option::is_none"`; those fields
  are now optional but non-null, with negative null fixtures;
- recursive Nickel record name resolution shadowed the raw
  `IsSha256Digest`/`IsMantleReference` predicates with sibling contract fields;
  the raw predicate bindings now have unambiguous names and a positive/negative
  callable-predicate regression test.

Current integrated evidence:

```text
$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --generate
machine schema contract generation: PASS (15 contracted, 44 classified)

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
machine schema contract self-test: PASS

$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (15 contracted, 44 classified)

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo test -q -p mantle --bin mantle machine_contract -- --nocapture
running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1355 filtered out; finished in 0.00s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo test -q -p mantle --test machine_schema_contracts -- --nocapture
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.59s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo test -q -p mantle --bin mantle oci_projection::tests::golden_reports_cover_full_minimal_and_both_import_states -- --exact --nocapture
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1360 filtered out; finished in 0.01s

$ nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-registry-target cargo check -q -p mantle --bin mantle
(exit 0)

$ nix develop -c rustfmt --check --edition 2024 src/machine_contract_producer_tests.rs src/oci_projection.rs scripts/check-machine-schema-contracts.rs scripts/machine_schema_contracts/mod.rs scripts/machine_schema_contracts/model.rs scripts/machine_schema_contracts/registry.rs scripts/machine_schema_contracts/schema.rs scripts/machine_schema_contracts/instance.rs scripts/machine_schema_contracts/render.rs scripts/machine_schema_contracts/freshness.rs tests/machine_schema_contracts.rs tools/tracey_refs.rs
(exit 0)

$ git diff --check
(exit 0)
```

The Cargo commands above ran through `nix develop -c env` with the isolated
`CARGO_TARGET_DIR` shown. Existing workspace warnings were captured separately
and are not promoted to clean-lint evidence.

## Current-tree lifecycle validation

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 15,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 39,
  "valid": true
}

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=45cc9a0cfdfc45f3bc877359c5ae077f23ba3879affaf944030c5994edc7e4c1
receipt_hash=8ffb021f8dae691a0f12e82b7f72e0f128e5a1d47a47f0328c1175f78570da8a
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=29507ac028708bd22f00bf3715ab66d86200943397d8869374166eb0b24ef0d4
receipt_hash=ac5a2600fa563c5dd57bc0da78ff9f9d4c4ff4dfbda1f51ed4da5f90d89c6132
issues=[] valid=true verdict=PASS

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json
input_hash=5092e48537e5e7b77051334dd1701bacd410892c0786a8f2226e7349fe8d5a2f
receipt_hash=7c08573032e431a025e8625923bec1314e813f8e379bcf828be488007cc757ca
issues=[] valid=true verdict=PASS
```

## Accepted-spec sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- sync expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json
dry_run=true blocked=false mutated=false reasons=[]
input_hash=e4306975864926daaa3770d02b3c69436122fe10a86d181fa768ac989ba2c91a
receipt_hash=52ef4b9969638e9e9b2b282aefc1cd6e0e458f6b86dbf0f926f02ae496f1811f

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- sync expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json --execute
dry_run=false blocked=false mutated=true reasons=[]
input_hash=e4306975864926daaa3770d02b3c69436122fe10a86d181fa768ac989ba2c91a
before_manifest_hash=b3ebaf18e2fb5d450012465c5990918f7a66e4c93a8053c9139e41f4d5f982b7
after_manifest_hash=ea12c42c960cb5aeae96a7323d5113884464a41a9ea26577f90604c3cb1f4214
mutation_manifest_hash=c4e621239ef4077168d08c7d45a49c696a2f9cb7b4e081f03bc8257f99c434fd
receipt_hash=d3e338a8b25ef4a19386c5628d9fbed507a5c4e8466066b25f508d2e7f3a44a1
```

The sync created `cairn/specs/machine-artifact-contracts/spec.md`; inspection
confirmed that every delta requirement and scenario is present in the accepted
spec.

## Archive and post-archive validation

```text
$ CAIRN_ARCHIVE_DATE=2026-07-12 nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- archive expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json
dry_run=true blocked=false mutated=false reasons=[]
input_hash=e4306975864926daaa3770d02b3c69436122fe10a86d181fa768ac989ba2c91a
receipt_hash=a0952f3ce269887ec8234bc0755ead349f724bdfa6cf5caf0c4983ae1365f52c

$ CAIRN_ARCHIVE_DATE=2026-07-12 nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- archive expand-machine-artifact-contract-registry --root . --policy cairn-policy/generated/cairn-policy.json --execute
dry_run=false blocked=false mutated=true reasons=[]
before_manifest_hash=e287b7e23ff18bfd96d907278d1e81535997e5a3612785895b7d7bf28f34a461
after_manifest_hash=fc12db8d41bd8586f177625ede64efae85826bdc2d6f1cbb871457b0288afaa3
mutation_manifest_hash=b579a2e641b87df2f218ffff536e97331c858068417c79b2cce7765334cadda8
receipt_hash=c37bcc631423f91324e9bb754d6ed692a15970ef8a7acc785952c1e984fcf9d4

$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 14,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 39,
  "valid": true
}
```

The archive was created at
`cairn/archive/2026-07-12-expand-machine-artifact-contract-registry` with the
same content hashes recorded for every moved lifecycle artifact.

## Claim boundary

Contract conformance proves only the declared JSON shape, bounds, linkage, and version policy. It does not prove build correctness, cache trust, reproducibility, release eligibility, attestation truth, provenance, deployability, or the truth of facts carried by a conforming artifact.
