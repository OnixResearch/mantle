# Validation Evidence

## Scope

This change consumes these exact producer revisions:

- Cairn `d953fe11ab620f3a42bdad1db51bc7672dd29824`
- Valence `6ab37aa34c9f81da812d6b42f2f06b7ac2d7e214`

The typed integration receipt is
`fixtures/content-bound-requirements/integration-receipt.ncl`. Its generated
JSON projection binds the frozen fixture paths and BLAKE3 values.

## Focused validation

These checks passed on 2026-07-27:

```text
cargo fmt --all --check
cargo test -p crunch-release-core
cargo test -p mantle --bin mantle content_bound -- --nocapture
cargo test -p mantle --test release_cli release_verify_strict_requirement_coverage_rejects_legacy_only_bundle -- --nocapture
cargo clippy -p crunch-release-core --all-targets --no-deps -- -D warnings
cargo clippy -p mantle --bin mantle --tests --no-deps -- -D warnings
cargo check -Zbuild-std=core,alloc -p crunch-release-core --target wasm32-unknown-unknown
nix run .#tigerstyle -- check -p crunch-release-core
nix run .#tigerstyle -- check -p mantle
nix build .#checks.x86_64-linux.content-bound-requirement-evidence -L
cairn policy export --source cairn-policy/default.ncl --output cairn-policy/generated/cairn-policy.json --check
cairn traceability coverage --root . --profile content-bound-release-requirements --json
```

The focused trace profile reported three requirements and three referenced
requirements. Its verdict was `pass`.

The Nix check type-checked the Nickel receipt, compared the generated JSON, and
recomputed each recorded fixture BLAKE3 value.

## Positive and negative coverage

Positive tests cover canonical ordering, frozen Cairn and Valence fixtures,
exact-byte shell measurement, release and binary binding, and optional migration
behavior.

Negative tests cover wrong repositories and revisions, stale registries and
receipts, digest-domain substitution, duplicate coverage, missing source or test
coverage, missing and cross-repository producer receipts, weakened non-claims,
unsafe capability paths, bundle tampering, strict legacy-only releases, and
failure without final bundle publication.

## Broader workspace results

`cargo test --workspace` stopped on the existing `crunch-eval` embedded-stdlib
mismatch. The repository has tracked
`lib/artifact-auth-cutover-receipt.ncl`, but the embedded list does not include
that file. This failure is outside this change.

`cargo test --workspace --exclude crunch-eval` passed 1,612 Mantle binary tests
and then hit `ExecutableFileBusy` in
`receipt_bound_c_compiler_alias_rewrites_response_file_runtime_inputs`. A
single-thread rerun of that exact test passed. No content-bound test failed.

## Octet result

The pinned Octet command inspected the touched content-bound core and shell
paths:

```text
nix run path:/home/brittonr/git/OnixResearch/octet#cargo-octet -- check
```

It later stopped on this unrelated existing diagnostic-path error:

```text
invalid Cargo diagnostic path: capability locator `crates/crunch-eval/src/../../../lib/remote-builders.ncl` contains `..`
```

Focused Tiger Style checks for both touched packages passed. The Octet failure
does not provide positive release evidence, and this change does not hide it.

## Adversarial review

A secondary local review proposed timestamp freshness and BLAKE3 collision tests.
Those proposals exceed the stated identity-and-linkage claim and were not
adopted. The review also prompted an added same-repository producer-receipt
invariant and negative tests in both core and shell code.

## Lifecycle gates

Cairn validation passed. Proposal, design, and tasks gates each reported `PASS`
with no issues. The focused traceability profile also reported `pass` with
three of three requirements referenced.

## Post-archive validation

- Cairn validation reported `valid: true` with no issues.
- The focused traceability profile reported `pass`, with three of three
  requirements referenced and no missing or dangling references.
- The focused trace receipt hash is
  `da151f148259f237b59fbb48a942a15ecdcb7e04f927f7b6a3d331c7fcbaf671`.

## Claim boundary

Passing checks prove exact supplied identity, bounded measurement, and declared
linkage only. They do not prove registry freshness beyond supplied bytes,
revision authenticity, requirement satisfaction, source correctness, test
truth, producer authority, runtime safety, or release eligibility.
