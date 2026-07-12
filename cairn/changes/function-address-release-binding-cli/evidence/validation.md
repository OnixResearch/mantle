# Validation evidence — function-address release binding CLI

Date: 2026-07-12

## Claim boundary

This evidence supports deterministic Mantle CLI binding of verified bundle-local function-address artifacts to source/binary identities, exact separation of artifact-byte and logical receipt identity domains, bounded no-follow public-envelope reads, and direct Cairn field consumption for the supplied fixtures. It does not prove function semantics, Rust or compiler correctness, Kamacite adapter correctness, Valence verifier soundness, whole-program safety, or release eligibility.

## Implemented boundary

- `mantle release function-address-bind` verifies a release-evidence bundle, selects declared external-evidence paths and a declared release binary, reads only public identity-envelope fields, and delegates policy/linkage decisions to `crunch-release-core`.
- Selected Valence and optional Kamacite files are opened through the release capability with component-wise no-follow access and a one-MiB bound. Reopened bytes are re-hashed against the verified manifest before parsing, closing post-verification replacement races.
- The core keeps bundle JSON byte digests in `verification_summary.*_receipt_digest_blake3` and projects upstream logical `receipt_hash` identities through `verification_summary.*_receipt_hash_blake3` into Cairn's legacy top-level `*_receipt_digest` fields.
- Valence's optional `kamacite_receipt_hash` must exactly equal the selected Kamacite logical `receipt_hash`; absent and partial optional metadata fail closed.
- Canonical output is synchronized through a same-directory temporary file and committed with no-clobber persistence. A structurally complete policy rejection may leave a deterministic `FAIL` receipt while returning non-zero; missing, stale, malformed, or replaced artifact bytes produce no receipt.
- This package corrects the v1 identity-domain projection before the operator-facing CLI establishes the receipt as a produced compatibility surface. The checked positive fixture's canonical inner `receipt_hash` is `9b571c22a2d4d5b9dc434c32565faa919d6a2dc862d5d0e318717df68d7b4a03`; the archived schema package's earlier fixture hash remains historical pre-CLI evidence rather than a current compatibility claim.

## Deterministic validation

All Cargo checks used isolated target directories and cleared the ambient Rust compiler wrappers where applicable.

- Pueue task `785`: `cargo test -p crunch-release-core` passed with `176 passed; 0 failed`; focused `cargo clippy -p crunch-release-core --all-targets --no-deps -- -D warnings` also passed.
- Pueue task `704`: direct installed-rustup `cargo check -p crunch-release-core --target wasm32-unknown-unknown` passed after supplying the documented clang PATH; the log ended with `Finished dev profile`.
- Pueue task `679`: machine-contract generation, self-test, freshness check, and the four `machine_schema_contracts` integration tests passed. The registry remained `16 contracted, 45 classified`.
- Pueue task `767`: six function-address binary/parser/read-binding tests passed, and the complete `release_cli` integration suite passed with `124 passed; 0 failed`.
- Pueue task `772`: the six CLI binding tests passed with `6 passed; 0 failed`; the required-mode test wrote `/tmp/mantle-function-address-cli-cairn-binding-final.json` from the real CLI.
- Pueue task `795`: root binary plus `release_cli` Clippy passed with `-D warnings` after explicitly allowing existing unrelated root-package lint debt. No allowance targets the function-address implementation.
- Pueue task `769`: the broad root Tiger Style rail retained unrelated existing findings, but its transcript contains no finding for `function_address_binding_cmd.rs`, the function-address parser tests, or the touched CLI fixture helpers.
- Pueue task `820`: focused Rust formatting checks for the touched core, CLI shell, parser root, and integration test plus `git diff --check` all passed.

The checked negative matrix covers missing sidecar metadata, wrong role, wrong schema, unsupported claim scope, source mismatch, overclaiming non-claims, stale artifact bytes, stale logical Valence-to-Kamacite linkage, no-clobber output, and policy `FAIL` receipt preservation. Shell unit tests additionally prove both acceptance of unchanged reopened bytes and rejection of post-verification replacement before parsing.

## Direct Cairn consumption

Pueue task `781` passed the real CLI-generated receipt directly to Cairn:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- release-readiness \
  --root . \
  --policy cairn-policy/generated/cairn-policy.json \
  --function-address-valence tests/fixtures/function-address-release-binding/valence-receipt.valid.json \
  --function-address-mantle /tmp/mantle-function-address-cli-cairn-binding-final.json \
  --function-address-kamacite tests/fixtures/function-address-release-binding/kamacite-receipt.valid.json
```

The `function_address` component reported:

```json
{
  "disposition": "present",
  "issues": [],
  "kamacite_receipt_hash": "0303030303030303030303030303030303030303030303030303030303030303",
  "mantle_binding_hash": "f01e4c21f65792354a5d1fdd6c071c721dc3e92070a12d48eea26a37ffc7a88b",
  "passed": true,
  "valence_receipt_hash": "0202020202020202020202020202020202020202020202020202020202020202"
}
```

The aggregate readiness receipt remained false only because independent workspace `traceability_coverage` and `mcp_agent_smoke` checks failed. This change claims only the passing function-address component.

## Bounded adversarial review

Goal: falsify bundle-to-receipt identity continuity, optional-link completeness, claim boundaries, deterministic output, and direct Cairn compatibility. Completion required a concrete counterexample or passing deterministic check; model agreement and task boxes were excluded. The budget was three mechanism families, one local advisory-model pass, one manual synthesis round, and focused repository checks.

| Family | Mechanism | Result |
|---|---|---|
| Identity-domain audit | Compare every top-level Cairn link with logical-envelope identities while retaining manifest byte digests separately | Validated by core tests, machine invariants, checked fixture parity, and direct Cairn consumption. |
| Filesystem continuity audit | Attempt replacement between bundle verification and selected identity read | Found a real mixed-time TOCTOU seam. Fixed by read-time BLAKE3 comparison to the verified manifest and positive/negative production-read tests. |
| Claim/optional-link audit | Try partial Kamacite metadata, stale Valence links, undeclared paths, role/schema drift, and semantic promotion | Fail-closed tests passed; no surviving semantic-promotion path was found. |

The local advisory model proposed two candidates: undeclared extra bundle bytes and undeclared function claims. Both were falsified against the completion contract: this receipt does not claim exhaustive bundle membership, and neither shell nor core accepts or parses function records. The model produced no concrete counterexample for the logical/artifact split. Repository validators remained authoritative.

Terminal result: validated for the bounded identity/linkage contract after fixing the TOCTOU seam. Residual uncertainty is limited to unproven upstream semantics and unrelated broad workspace lint/Tracey debt, both outside this change's claim.

## Lifecycle gates

Pueue task `819` ran repository-policy validation and proposal, design, and tasks gates serially. Validation reported 11 active changes, 38 validated specs, no issues, and `valid: true`. Every stage gate returned no issues, `valid: true`, and `verdict: PASS`. Pueue task `828` reported `policy fresh` for `cairn-policy/generated/cairn-policy.json`.

The broad Tracey command in task `822` still reports established repository-wide missing coverage plus active-change references as dangling before sync. It reports no implementation gap specific to this package's deterministic checks; this output is not claimed as a passing Tracey rail. The package-level Cairn gates above are the pre-archive lifecycle evidence.

## Non-claims for failed exploratory commands

The first Nix-shell wasm check used a toolchain without the installed wasm target, and the first direct-rustup retry lacked clang on PATH; neither is counted as evidence. An early Kache-backed root Clippy run hit a compiler ICE in an unrelated dependency, and later broad Clippy attempts exposed unrelated root-package lint debt; only task `795` is claimed. Initial parser tests overflowed the default libtest worker stack while constructing the repository's very large Clap graph; the final parser tests use a named bounded test thread and task `767` is the claimed result.
