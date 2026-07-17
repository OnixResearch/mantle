# Validation evidence

## Baseline

Pueue task `79` established the pre-change registry baseline: seven core tests and four public CLI integration tests passed.

## Functional core and trust invariants

`src/oci_registry.rs` now owns deterministic policy normalization, domain-separated policy/public-key BLAKE3 identities, canonical signed statements/documents, signature artifact construction/inspection, detached Ed25519 verification, full-key revocation, required-signer checks, distinct-key threshold, and v2 receipt verification. Registry routing and secret/path inputs remain outside signed identity.

The selected design survived an adversarial review over three materially distinct mechanisms recorded in `evidence/baseline.md`. A local VibeThinker review suggested explicit key-name collision, quorum, replay, substitution, ordering, and receipt checks. Repository invariants remain authoritative: same-name rotation is permitted only for distinct full-key identities; duplicate full key material is rejected; quorum counts distinct key-material BLAKE3 identities; repository replay is rejected by explicit verifier policy; signature ordering is canonical and non-canonical permutations are rejected.

Task `65` passed eleven focused core tests. Positive coverage includes one-key and two-key signed statements plus same-name key rotation with distinct key identities. Negative coverage includes insufficient quorum, duplicate key material under another name, duplicate signatures, unknown/revoked keys, wrong domain/digest pair, signature subject/annotation drift, receipt tampering, and collection/accounting bounds.

## Shell and public CLI ordering

`src/oci_registry_shell.rs` loads typed Nickel policy, verifies repository authorization before network I/O, publishes metadata and signature companions before the image tag, and re-reads all three manifests by immutable digest. Pull resolves all three tags, downloads only the descriptor-bound signature document, validates policy/signatures, then downloads image/metadata content blobs and invokes ordinary OCI import.

Task `65` passed:

```text
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1551 filtered out
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The typed Nickel test accepts the valid contract and rejects a string threshold. Public CLI cases prove trusted exact round trip and fail-closed image/signature tag drift, metadata tamper, unknown signer, invalid signature bytes, wrong repository policy, revoked policy, denied credentials, interrupted final image publication, and idempotent blob reuse. Unknown/bad-signature tests inspect the loopback server and prove only the signature document blob was fetched before rejection; no layout, state, import report, or pull receipt exists.

## Contract and documentation rails

Task `65` regenerated and checked the machine registry:

```text
machine schema contract generation: PASS (18 contracted, 47 classified)
machine schema contract check: PASS (18 contracted, 47 classified)
```

The Rust-owned v2 push/pull DTOs, JSON schemas, generated Nickel review contracts, Rust-produced golden fixtures, adversarial negative sets, inventory version/non-claim policy, and BLAKE3 freshness agree. New fields bind signature manifest, trust domain, policy identity, verified signer names, and verified full-key identities without exposing key material or paths.

Task `65` also passed 17 example-inventory tests, 11 workflow-gallery tests, and 10 focused embedded-stdlib tests. The checked gallery policy contains only a public demonstration key; matching private material is absent. README, runbook, operator docs, catalog, workflow metadata, and ADR 0029 agree on three immutable digests and the bounded signature claim.

## Structural quality

Task `63` passed focused root-package Clippy with `-D warnings` and the complete configured first-party Tiger Style rail after decomposing compound conditions, long receipt validation, ambiguous same-type interfaces, and key-identity handling.

## Claim boundary

Successful evidence authenticates only the immutable image/metadata digest pair under supplied local policy. It does not prove registry authorization, transparency, revocation freshness, tag immutability, arbitrary-registry compatibility, artifact correctness, exactly-once publication, upload resumption, kernel compatibility, bootability, deployability, or release eligibility.

Full first-party quality, dependency policy, lifecycle sync/Tracey, archival, and push remain V3/V4 work.
