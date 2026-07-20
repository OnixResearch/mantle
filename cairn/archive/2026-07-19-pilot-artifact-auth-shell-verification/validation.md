# Validation evidence

## Baseline and isolation

Clean revision `7ed004dc85211dd8aa406161aade35b369b329a0` passes the focused `crunch-action-result-core` artifact-auth suite and `crunch-build` action-result suite; the preceding full clean-commit `nix flake check` also passed. The original Mantle worktree retains unrelated bootstrap/source-seed files. This pilot runs in `/home/brittonr/git/OnixResearch/mantle-artifact-auth-shell` on its own branch and must not include or modify those files.

The portfolio frame, registry, false-completion cases, audit risks, and budgets are recorded in `design.md`. Action-result shell integration is the surviving route; OCI registry and release-attestation routes are rejected because they sign different subjects and own different authority.

## Focused implementation evidence

The pure core maps the exact action-result subject, ordered output/optional OCI parents, publication-policy context, producer, key label, and full public-key identity without consuming either legacy or standalone cryptographic outcomes. The `crunch-build` shell validates the exact signed record and admitted decision, verifies one unambiguous legacy signature over `result_ref`, requires producer/key/trust-basis/currentness/full-key agreement, separately signs canonical standalone bytes, recomputes all carrier identities, and independently invokes pinned `artifact-auth-ed25519`.

Positive round-trip and adversarial tests pass for malformed mapping, legacy authorization rejection, missing trust basis, tampered or ambiguous legacy signatures, producer substitution, unknown/revoked currentness, wrong same-label full key, malformed/relabeled public-key carrier, detached legacy-signature reuse, changed statement preimage, malformed/tampered standalone signatures, carrier/signature-encoding/authorization drift, both decision-drift directions, and unrelated-failure false parity. Focused rustfmt, strict no-dependency Clippy, repository first-party Clippy, and Tiger Style pass.

The advisory VibeThinker review requested additional authorization-substitution and alias checks. Audit found a concrete same-label different-key path where standalone evaluation could have proceeded while legacy authorization was false. Evaluation now rejects `LegacySignatureInvalid` before compatibility, binds producer/key identity in common validation, and adds ambiguous-signature, producer-substitution, malformed-key, relabeled-key, and same-label wrong-full-key tests. The advisory suggestion to perform live currentness refresh is intentionally not implemented: network/key-store/currentness effects remain product-owned and their absence blocks authority admission rather than widening this pilot.

## Full validation

Implementation commit `63affce0` passes the full first-party quality wrapper in repository-local `target/artifact-auth-shell`, native Cairn validation and task gates, and full `nix flake check -L` on `x86_64-linux`. Nix's `mantle-nextest` partition ran 4,007 tests with 4,007 passing and eight skipped. The exact Cargo/Nix source assertion admits only `artifact-auth-core` and `artifact-auth-ed25519` at revision `799459346d5416fbd7b9f55840a7371441b55afa`; Cargo generated `Cargo.lock`, and no `flake.lock` update was required because the existing immutable source input did not change.

The first full-wrapper attempt hit a Rust compiler incremental-cache panic while forcing `crunch-delta` in shared `~/.cargo-target`. The identical wrapper passed after setting the repository-local target directory already used by focused checks; no source, test, lint, or gate was disabled.

## Cross-consumer authority evaluation

Published Molten `a580318361288a6be716e4a5ae229a5e864e7192`, published Valence `4f27958469e5`, and Mantle `63affce0` all provide a separate exact standalone signature, product-specific signing guard, independent pinned verifier, adversarial preimage/key/currentness evidence, legacy authority, standalone non-authority, and rollback. Their product identities and authorization evidence are intentionally different and explained: Molten uses capability/key-handle facts, Valence uses Radicle actor/operation authorization, and Mantle uses admitted action-result plus detached record-signature facts.

Authority admission is rejected. None of the three archives proves a live durable operation with fresh trust discovery, currentness/revocation refresh, product-owned publication/persistence, and subsequent consumer verification. Mantle additionally receives key generation/currentness as supplied observations because its Nix-compatible key adapter has no live generation/currentness authority. Synthetic and Nix test execution cannot fill those gaps. No authority-admission Cairn change is created; every consumer remains `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`.
