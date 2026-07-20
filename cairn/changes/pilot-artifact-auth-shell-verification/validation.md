# Validation evidence

## Baseline and isolation

Clean revision `7ed004dc85211dd8aa406161aade35b369b329a0` passes the focused `crunch-action-result-core` artifact-auth suite and `crunch-build` action-result suite; the preceding full clean-commit `nix flake check` also passed. The original Mantle worktree retains unrelated bootstrap/source-seed files. This pilot runs in `/home/brittonr/git/OnixResearch/mantle-artifact-auth-shell` on its own branch and must not include or modify those files.

The portfolio frame, registry, false-completion cases, audit risks, and budgets are recorded in `design.md`. Action-result shell integration is the surviving route; OCI registry and release-attestation routes are rejected because they sign different subjects and own different authority.

## Focused implementation evidence

The pure core maps the exact action-result subject, ordered output/optional OCI parents, publication-policy context, producer, key label, and full public-key identity without consuming either legacy or standalone cryptographic outcomes. The `crunch-build` shell validates the exact signed record and admitted decision, verifies one unambiguous legacy signature over `result_ref`, requires producer/key/trust-basis/currentness/full-key agreement, separately signs canonical standalone bytes, recomputes all carrier identities, and independently invokes pinned `artifact-auth-ed25519`.

Positive round-trip and adversarial tests pass for malformed mapping, legacy authorization rejection, missing trust basis, tampered or ambiguous legacy signatures, producer substitution, unknown/revoked currentness, wrong same-label full key, malformed/relabeled public-key carrier, detached legacy-signature reuse, changed statement preimage, malformed/tampered standalone signatures, carrier/signature-encoding/authorization drift, both decision-drift directions, and unrelated-failure false parity. Focused rustfmt, strict no-dependency Clippy, repository first-party Clippy, and Tiger Style pass.

The advisory VibeThinker review requested additional authorization-substitution and alias checks. Audit found a concrete same-label different-key path where standalone evaluation could have proceeded while legacy authorization was false. Evaluation now rejects `LegacySignatureInvalid` before compatibility, binds producer/key identity in common validation, and adds ambiguous-signature, producer-substitution, malformed-key, relabeled-key, and same-label wrong-full-key tests. The advisory suggestion to perform live currentness refresh is intentionally not implemented: network/key-store/currentness effects remain product-owned and their absence blocks authority admission rather than widening this pilot.
