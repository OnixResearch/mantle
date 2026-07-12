# Machine artifact contracts

Mantle classifies every annotated public machine-readable JSON family in
[`schemas/machine-contracts/inventory.ncl`](../schemas/machine-contracts/inventory.ncl).
The five classes are:

- `contracted`: stable review/consumer boundary with schema, generated Nickel
  contract, positive and adversarial fixtures, version policy, and BLAKE3
  freshness;
- `compatibility`: existing public behavior retained without freezing a new
  exact aggregate schema;
- `internal`: Rust implementation data not promised to external consumers;
- `debug`: bounded diagnostics unsuitable for readiness or automation; and
- `external`: shape owned by user data, a protocol, or another producer.

A Rust producer uses a `// machine-artifact-public: <surface-id>` annotation.
The checker fails if an annotation has no registry decision or if a registered
producer annotation disappears or moves outside its declared owner sources. It
also scans root-package Rust sources that serialize JSON through the standard
string/vector helpers (compact or pretty) and requires each such module to
belong to at least one classified producer family.
Planned remote-attempt, observability, resumable-transfer, and Wasm-component
surfaces are classification-only entries: registration does not claim those
active changes are implemented.

## Authority flow

Authority is one-way:

1. Rust DTOs own emitted runtime facts.
2. A checked JSON Schema snapshot records the exact supported review
   projection. The checker parses the exact Rust owner syntax, compares root
   fields, requiredness, and resolvable scalar/container categories, and
   accounts for `serde(rename)`, `skip_serializing`, and `skip_serializing_if`.
   It rejects comment-spoofed declarations, conditional fields, flattening,
   custom field serializers, and unsupported struct-level serializer rewrites;
   producer tests also compare Rust serialization with positive fixtures.
3. The pure Rust generator accepts only a bounded schema subset and renders a
   Nickel contract from it.
4. Nickel evaluates only in tests as typed review evidence. Product commands do
   not load Nickel contracts or execute a Nickel validator before emitting JSON.

Unsupported schema keywords, multi-type unions, `$ref` siblings, missing or
non-root-definition references, recursive schemas, misplaced constraints,
untyped arrays, permissive object tails, fractional collection bounds, and
unnamed bounds fail closed. Contract artifact paths are normalized and confined
to `schemas/machine-contracts/`. The generator never falls back to `Dyn`.

The shared generated-contract vocabulary covers exact versions, closed enums,
integer and collection bounds, BLAKE3 and protocol-required SHA-256 syntax,
safe references, redaction-safe text, and declared cross-field invariants.
Every numeric bound carries an `x-mantle-bound-name` and is emitted as a named
Nickel `let`, so generated predicates contain no unexplained limit literals.
String bounds also carry `x-mantle-length-unit = "utf8-bytes"`; the Nickel
predicate computes exact UTF-8 byte length through unpadded Base64 length rather
than Nickel's grapheme-counting `std.string.length`.

## Validation and generation

Check committed state:

```bash
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
```

Run adversarial checker self-tests:

```bash
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
```

Regenerate contracts and BLAKE3 freshness fields after an intentional owner,
schema, fixture, policy, consumer, or non-claim change:

```bash
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --generate
```

Generation is the only mutating checker mode. Review the resulting schema,
contract, fixture, and inventory diff together. A prior version may be listed
as supported only when the entry also names a deterministic converter and
positive/negative migration fixtures; otherwise consumers reject it.

## Claim boundary

Contract conformance proves only the JSON shape and declared linkage
invariants. It does **not** prove build correctness, cache or signer trust,
reproducibility, release eligibility, attestation truth, source provenance,
runtime behavior, frontend semantics, or deployability. Those claims remain
with their owning build, trust, attestation, release, and lifecycle evidence
rails.
