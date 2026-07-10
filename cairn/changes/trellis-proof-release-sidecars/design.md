## Design

Mantle treats Trellis proof evidence as a profile of the generic opaque sidecar binding. The release validation core receives typed metadata that links the release source archive and binary to Valence-validated Trellis proof evidence. The core checks hashes, roles, profile version, claim scope, policy hashes, and non-claims, but does not parse proof IR, verifier logs, Verus source, or Preserves internals.

### Decisions

- **Opaque proof sidecar.** Mantle binds proof evidence metadata only; Valence owns proof evidence validation.
- **Release linkage is explicit.** Source archive and release binary identities are typed links in the binding receipt.
- **Proof role is carried, not proven.** Mantle preserves reference-only/candidate/accepted role metadata from Valence without deciding verifier soundness.
- **Claim scope is release-local.** Binding proves only that the release carries declared proof evidence sidecars for declared artifacts.
- **JSON remains projection.** Compatibility sidecars are accepted only when bound to canonical Kamacite envelope identity.

### Validation shape

Positive fixtures cover required accepted proof evidence and optional recorded-only proof evidence. Negative fixtures cover missing canonical envelope hash, stale Valence hash, source/binary mismatch, unsupported proof profile version, wrong role, unsupported claim scope, projection drift, malformed BLAKE3, and overclaiming release text.
