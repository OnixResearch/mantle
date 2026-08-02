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
Registry entries can exist before a producer workflow is complete. Thus,
classification does not prove implementation. The Rust client and server own
current resumable-transfer evidence. The validation rails are in
[`remote-transfer.md`](remote-transfer.md).

Foreign import command reports use the `import.command-reports` compatibility
family. A successful planning report now contains the versioned
`mantle-foreign-executable-plan-v1` artifact. Its Rust owner is
`src/foreign_executable_plan.rs`.

The plan binds import identity, roots, native units, exact path maps, source requirements, profiles, diagnostics, route, and non-claims.
Its BLAKE3 identity covers these fields.
`cache-only-preserve-v1` identifies exact-path cache observation, not local build authority.
This classification does not prove source availability, scheduler execution, store admission, realization, or output trust.

The same family includes `mantle-foreign-realization-receipt-v1`.
This receipt binds the plan, import receipt, source bundle, profiles, build report, PathInfo facts, and non-claims.
It also binds cache-closure policy, ordered closure members, signatures, NAR facts, references, depths, and transfer dispositions.
Contract conformance does not prove package correctness, evaluator parity, provenance, reproducibility, or release eligibility.

The family also includes `mantle-foreign-provenance-audit-v1`. This audit binds
one realization receipt to selected roots, signed PathInfo, castore identities,
path maps, profiles, scanner policy, limits, observations, findings, and
non-claims. Payload observations can classify bounded gzip and zstd streams.
Reference observations record normalized store suffixes that stay inside one
store root. A passing disposition can report `provenance-audited` for this
bounded scope. It does not prove dynamic behavior or package correctness.

OCI registry push/pull receipts are contracted separately from local OCI
export/import reports. The push receipt binds the image and subject-metadata
manifest SHA-256 values needed for immutable pull. The pull receipt binds both
expected/resolved manifest pairs to the reconstructed layout/projection BLAKE3
and ordinary OCI import receipt. Their `credential_mode` records only
`anonymous` or `explicit-bearer-file`; credential paths and bytes are forbidden
from the DTOs and negative fixtures. Contract conformance does not grant
registry trust, authorization, tag immutability, signature verification, or
release eligibility.

Full-source provider promotion has a contracted
`mantle-full-source-provider-admission-v2` report. It binds the normalized
provider tree, provider metadata, and complete materialized source closure to
independently supplied BLAKE3 values after positive and rejection runtime smoke.
Contract conformance does not prove compiler correctness, bootstrap-seed
correctness, independent rebuild agreement, release reproducibility,
deployment success, or full Cargo compatibility.

Fresh-clone source hydration has its own contracted
`mantle-self-build-source-hydration-v1` report. It binds the externally checked
source-bundle manifest BLAKE3, hydrated vendor BLAKE3, legacy provider archive
BLAKE3, import counts, and pin state. Checkout paths, temporary staging paths,
Cargo caches, and credentials are excluded. Report conformance proves neither
that the expected digest came from a trusted publisher nor that a later
fixed-point self-build succeeds.

The heavier hydrated proof emits the separate contracted
`mantle-hydrated-fresh-clone-fixed-point-v1` report. It links the hydration
receipt BLAKE3 to the independently supplied source manifest, source-state
identity, staged source store name, provider/platform/proof mode, enforced
stage policies, zero live-fetch counts, and stage binary BLAKE3 values. It
excludes checkout, source-state, cache, credential, executable, and temporary
paths. `fixed_point: true` means only that the recorded stage1/stage2 binary
digests match under this bounded proof; it does not promote compiler, seed,
release, independent-rebuild, deployment, or full-Cargo claims.

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
The `field-equals-when` invariant expresses closed discriminator/value pairs,
so versioned profile roles cannot be combined with a sibling profile's schema.
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
