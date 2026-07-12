# ADR 0015: Keep machine artifacts Rust-owned and generate Nickel review contracts

## Status

Accepted

## Context

Mantle emits many public JSON reports and handoff artifacts. The original machine-contract pilot covered only the doctor report with a checker branch dedicated to that one shape. Expanding that pattern report by report would duplicate validation logic, leave other public outputs unclassified, and risk making Nickel evaluation a runtime dependency of Rust commands.

The public boundary needs three distinct authorities: Rust DTOs own emitted facts, language-neutral schemas make the serialized projection reviewable, and Nickel contracts let downstream reviewers inspect typed constraints. These authorities must not form a runtime cycle.

## Decision Drivers

- Preserve Rust serialization as the only runtime producer authority.
- Keep Nickel typed and reviewable without executing Nickel in product paths.
- Reject unsupported schema constructs instead of weakening them to `Dyn`.
- Make adding a contract a data change plus focused producer evidence.
- Bind owner source, schema, generated contract, fixtures, consumer policy, and non-claims against silent drift.
- Keep validation logic pure and deterministic, with filesystem mutation isolated to an explicit generation shell.

## Decision

Maintain `schemas/machine-contracts/inventory.ncl` as the typed inventory of every annotated public machine-JSON family. Each surface is classified as `contracted`, `internal`, `debug`, `compatibility`, or `external`. Contracted entries name one Rust owner, an exact schema snapshot, generated Nickel contract, positive and negative fixtures, consumers, version policy, validation commands, non-claims, and BLAKE3 freshness bindings.

Rust DTOs remain authoritative for emitted facts. Producer tests serialize representative Rust values and compare them with the registered positive fixtures. The checker statically compares each contracted root DTO's serialized field and requiredness sets—including serde rename and skip directives—with the schema root and binds all declared owner sources with BLAKE3, so owner changes require explicit contract review.

A pure Rust core validates a bounded JSON Schema subset and deterministically generates eager Nickel predicate contracts using a shared exact vocabulary for versions, closed records, enums, numeric and collection bounds, BLAKE3, safe references, redaction-safe text, and cross-field invariants. Its `field-equals-when` invariant represents closed discriminator/value pairs without admitting unsupported JSON Schema conditionals. Eager predicates return the original constant JSON value and avoid Nickel's delayed recursive-record contract thunks. Schema bound names become deterministic Nickel `let` declarations instead of unexplained limit literals. String bounds explicitly use UTF-8 bytes and derive byte length from unpadded Base64 length, avoiding Nickel's grapheme-counting string length. Unknown keywords, unresolved references, recursive definitions, permissive object tails, unnamed bounds, and ambiguous string-length units fail closed. Generation never emits `Dyn`.

The command shell only reads and writes files, scans annotated Rust sources, and reports deterministic issues. `--generate` is the sole mutation mode. Product commands do not load the inventory, schemas, contracts, or Nickel evaluator. Nickel evaluation occurs only in tests to prove the typed inventory and generated contracts accept Rust-serialized positives and reject adversarial fixtures.

All Mantle-owned freshness identities use BLAKE3. Protocol-owned hashes such as NAR SHA-256 remain unchanged when represented inside a contracted artifact.

## Alternatives Considered

### Execute Nickel contracts before emitting runtime JSON

Rejected because it adds evaluator authority and latency to product commands, creates a cyclic Rust-to-Nickel runtime boundary, and can make reporting fail for reasons unrelated to the underlying operation.

### Derive permissive contracts from arbitrary JSON Schema

Rejected because unsupported unions, conditionals, references, or object tails would silently weaken review. An exact bounded subset with a clear blocker is safer.

### Hand-maintain independent Nickel contracts

Rejected because schemas and contracts would drift independently and each new surface would require bespoke checker code.

### Contract every serializable implementation DTO immediately

Rejected because it would freeze internal detail. Complete classification is required now; stable contracts can migrate in bounded cohorts.

## Consequences

- Public producers must carry a `machine-artifact-public` annotation and a matching inventory decision.
- Contract schema changes require regenerated Nickel and freshness bindings plus positive and adversarial fixtures.
- Prior versions are rejected unless the inventory names a converter and migration fixtures.
- The bounded schema subset can block a desired shape until the core gains an exact, tested representation.
- Contract conformance proves shape and declared linkage only; it does not prove build correctness, trust, reproducibility, release eligibility, attestation truth, or deployability.
