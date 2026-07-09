## Design

Extend external release evidence with Preserves carrier rows: bundle path, schema label, producer repo, canonical BLAKE3 digest, evidence role, claim scope, payload opacity mode, optional adapter ID, and non-claims. The release verifier checks carrier identity and role/scope linkage. Profile-specific adapters can add bounded payload checks without becoming the default.

## Non-claims

A valid carrier row does not prove producer runtime semantics, evidence truth, authorization correctness, or Cairn/Valence acceptance unless separate receipts are also present and linked.
