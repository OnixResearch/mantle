# Validation evidence

## Baseline

Before core changes, native Cairn validation and proposal/design/tasks gates passed. `nix develop -c cargo test -p crunch-action-result-core` passed the current action-result unit, architecture, and documentation tests. Review of `crunch-build::action_result` and `crunch-action-result-core` established the existing result-ref preimage, detached signer labels, verified-signer facts, product admission diagnostics, and retained build/store/repository authority.

The reviewed standalone Mantle profile at artifact-auth revision `799459346d5416fbd7b9f55840a7371441b55afa` records source baseline `26c6880c08cd730c9e9420f92ea91a87212e492d`, shared domain/subject/parent/signer/full-key/revocation/threshold/Ed25519 fields, Nix-key and OCI compatibility extensions, and retained repository/registry/credential/signing/OCI/build/release authority.

## Exact source admission

Cargo and Nix resolve `ssh://git@github.com/OnixResearch/artifact-auth.git` at full revision `799459346d5416fbd7b9f55840a7371441b55afa`. `Cargo.lock` contains one `artifact-auth-core` package from that revision. `flake.lock` records the same SSH URL/revision and NAR hash `sha256-nEgz2FtVuDesX95yyxidp0vhjxL4INB6Ve8rkpLyJk0=`. Flake evaluation asserts exact Cargo/Nix revisions, lock uniqueness/source, and `MIT OR Apache-2.0` licensing. Crane vendors the exact flake input through its Git-checkout override instead of a sibling path. Cargo and Nix tooling generated both locks.

## Adapter and cutover result

`crunch-action-result-core::artifact_auth` is pure. It maps a validated result subject, output-object parents, publication-policy verifier context, explicit full-key/currentness observations, threshold, required signer labels, and an optional complete OCI/metadata SHA-256 pair. Every signer supplies a separate `CryptographicObservation`; legacy detached signatures and verified labels are never reused for standalone bytes.

Compatibility classifies distinct preimages, exact identities, decisions, mapped issue causes, mandatory non-claims, duplicate keys, missing labels, incomplete OCI pairs, and unrelated-failure false parity. Legacy authority and rollback remain true. Runtime authority is rejected because no Mantle shell currently signs/verifies exact standalone statements; `standalone_authority_admitted` is fixed false.

## Approach registry

| Family | Result |
| --- | --- |
| Treat Nix key labels as full identity | Rejected: labels cannot count as distinct full keys. |
| Reuse detached action-result signatures | Rejected: they cover a different preimage. |
| Move OCI/repository/build admission into artifact-auth | Rejected: it transfers Mantle authority. |
| Pure action-result adapter with explicit standalone observations | Selected: exact mapping and drift are testable while effects and product admission stay in Mantle. |

## Focused verification

Three positive/negative adapter tests pass. They cover valid action-result/OCI mapping, full-key and label identity, mandatory non-claims, signature tamper, unrelated rejection causes, false parity, duplicate-key and duplicate-label inflation, revoked currentness, missing required labels, malformed or incomplete OCI pairs, and retained repository/build/release authority. Focused rustfmt, strict all-target Clippy, the repository Tiger Style rail, first-party strict Clippy, all non-root first-party workspace lib/tests, and native Cairn validation pass.

The ordinary dirty-worktree gate also reached two failures in `tests/bootstrap_eval.rs` owned by the separate uncommitted `promote-full-source-seed-provider` work. The staged-source Nix check sees five related failures because Git flakes omit that change's untracked bootstrap source files. This adoption does not edit, stage, suppress, or claim those tests. Its focused action-result rail and all other first-party packages pass; a clean task-owned commit is required for the final Nix rerun.

These checks establish deterministic mapping over supplied observations only. They do not prove OCI truth, repository authorization, trust-root/currentness freshness, signing correctness, cache/build correctness, transport, deployment safety, or release eligibility.
