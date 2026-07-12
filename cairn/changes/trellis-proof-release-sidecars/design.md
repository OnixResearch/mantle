## Design

Mantle treats Trellis proof evidence as a profile of the generic opaque sidecar binding. The release validation core receives typed metadata that links the release source archive and binary to Valence-validated Trellis proof evidence. The core checks hashes, roles, profile version, claim scope, policy hashes, and non-claims, but does not parse proof IR, verifier logs, Verus source, or Preserves internals.

### Decisions

- **Opaque proof sidecar.** Mantle binds proof evidence metadata only; Valence owns proof evidence validation.
- **Release linkage is explicit.** Source archive and release binary identities are typed links in the binding receipt.
- **Proof role is carried, not proven.** The current profile preserves Kamacite `recorded-only` or `formal-proof-candidate` while requiring Valence `recorded_only`; candidate evidence is not promoted.
- **Claim scope is release-local.** Binding proves only that the release carries declared proof evidence sidecars for declared artifacts.
- **JSON remains projection.** Compatibility sidecars are accepted only when bound to canonical Kamacite envelope identity.
- **Accepted proof fails closed until authority exists.** Required mode cannot pass from `recorded_only`. A passing accepted-proof path needs Valence's proposed Trellis validator and authoritative receipt/profile vocabulary.

### Validation shape

Positive fixtures cover optional recorded-only evidence and preservation of a formal-proof candidate without promotion. Negative fixtures cover required-mode absence of accepted authority, missing canonical envelope hash, stale Valence artifact and logical hashes, source/binary mismatch, unsupported proof profile version, wrong roles, unsupported claim scope, projection drift, malformed BLAKE3, and overclaiming release text.

The originally requested required accepted-proof positive fixture and live stack smoke remain blocked. At Valence revision `7a027529dd4b7057cf52e86dc5b258f2a9541545`, active change `trellis-proof-evidence-profile` has every task unchecked, including validator registration, accepted-formal-proof role behavior, accepted fixtures, graph output, and final stack validation.
