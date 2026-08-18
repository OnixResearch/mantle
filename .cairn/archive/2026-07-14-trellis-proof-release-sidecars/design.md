## Design

Mantle treats Trellis proof evidence as a profile of the generic opaque sidecar binding. The release validation core receives typed metadata that links the release source archive and binary to Valence-validated Trellis proof evidence. The core checks hashes, roles, profile version, claim scope, policy hashes, and non-claims, but does not parse proof IR, verifier logs, Verus source, or Preserves internals.

### Decisions

- **Opaque proof sidecar.** Mantle binds proof evidence metadata only; Valence owns proof evidence validation.
- **Release linkage is explicit.** Source archive and release binary identities are typed links in the binding receipt.
- **Proof role is carried, not proven.** The profile preserves Kamacite `recorded-only` or `formal-proof-candidate`; only the exact candidate plus Valence `property` pair records accepted upstream authority.
- **Claim scope is release-local.** Binding proves only that the release carries declared proof evidence sidecars for declared artifacts.
- **JSON remains projection.** Compatibility sidecars are accepted only when bound to canonical Kamacite envelope identity.
- **Accepted proof fails closed without exact authority.** Required mode cannot pass from `recorded_only`, a `property` role attached to a recorded-only producer, or any unknown role pair. Valence commit `27b8b212` supplies the accepted-formal-proof semantics.

### Validation shape

Positive fixtures cover optional recorded-only evidence, preservation of a formal-proof candidate without promotion, and required accepted evidence for the exact Kamacite candidate plus Valence property pair. Negative fixtures cover required-mode absence of accepted authority, unauthorized role pairing, missing canonical envelope hash, stale Valence artifact and logical hashes, source/binary mismatch, unsupported proof profile version, unsupported claim scope, projection drift, malformed BLAKE3, and overclaiming release text.

The stack smoke runs Valence's focused accepted and negative Trellis profile tests at archived authority commit `27b8b212`, then runs Mantle's complete release-core suite and strict core Clippy. These checks preserve the boundary that Valence decides acceptance while Mantle validates only the measured release binding.
