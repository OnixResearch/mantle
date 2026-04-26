## Context

The current bootstrap chain already builds most tools from source, but it starts
from a reduced provider derived from a musl.cc binary tarball. Later stages
consume a normalized seed contract, which is the right seam for swapping the
provider without rewriting every bootstrap derivation.

## Goals / Non-Goals

**Goals**
- Define a versioned source-root manifest with explicit digests and provenance.
- Build a provider from source that satisfies `bootstrap/seed.ncl`.
- Keep provider-specific layout hidden behind the normalized seed contract.
- Gate the full-source claim on proof evidence.

**Non-Goals**
- Solve host-tool-free bootstrap execution in the same change.
- Claim bit-for-bit release reproducibility.
- Remove all tiny audited seeds if they are explicitly documented and hashed.

## Decisions

### 1. Treat the manifest as the root of trust

**Choice:** add a repo-owned source-root manifest that names every pre-provider
source, patch, digest, extraction rule, expected output, network trust root, and
remaining tiny-seed/bootstrap assumption trust note.

**Rationale:** reviewers need one bounded object to audit. Spreading root facts
across `.ncl`, shell snippets, and docs makes the bootstrap claim impossible to
verify mechanically.

**Manifest schema:** the versioned manifest contains these top-level lists:

- `artifacts`: name, URL or local path, digest algorithm/value, optional
  non-BLAKE3 reason, extraction rule, provenance, and patch references.
- `patches`: name, digest algorithm/value, optional non-BLAKE3 reason,
  provenance, and apply order.
- `network_trust_roots`: URL authority, digest/provenance for pinned metadata,
  and rationale.
- `trust_notes`: name, digest, provenance, scope, and rationale for any
  remaining tiny seed or bootstrap assumption.
- `expected_outputs`: normalized provider output name, kind, digest,
  provenance, and required contract role.

**Validation algorithm:** pure validation rejects unsupported versions, missing artifact extraction
rules, missing artifact/patch/output/network/trust-note digests, missing
provenance, trust notes missing `scope` or `rationale`, non-BLAKE3 digests
without an interoperability reason, artifact patch references not present in
`patches`, and provider dependency URLs not present in artifacts or
`network_trust_roots`. A separate provider-output validation pass rejects
emitted provider output roles not present in `expected_outputs`.

**Alternative:** document the source root only in README text.

**Why not:** prose cannot drive deterministic validation or proof reporting.

### 2. Keep `bootstrap/seed.ncl` as the provider contract

**Choice:** make the source-built provider satisfy the same normalized contract
as the current fetched provider.

**Provider construction:** the source-root provider is selected with
`crunch bootstrap --source-root <manifest>`. Phase 1 keeps host-tool-free
execution out of scope: the manifest may declare a bounded host POSIX shell,
`cc`/`c++`, `make`, archive tools, and patch application tool as `trust_notes`.
Those tools are remaining tiny seeds, not eliminated trust. The provider source
set is explicit and deterministic: Linux UAPI headers, musl source, binutils
source, GCC source/runtime source, and manifest-declared patches. Build phases
are fixed: validate manifest, fetch/read artifacts, verify digests, unpack by
manifest extraction rule, apply declared patches in order, build/install into a
staging root, normalize staging into the seed contract, write
`share/crunch-bootstrap/provider.json`, and validate the normalized output. The
provider build must not depend on or fetch the legacy
`https://musl.cc/x86_64-linux-musl-native.tgz` tarball; provider dependency
validation scans artifact URLs, network trust roots, provider metadata, and the
recorded dependency trace for that legacy URL/hash and fails if found.
`crunch bootstrap --fetch` remains the explicit seed-assisted legacy path;
`--fetch --source-root <manifest>` fails before network or build work starts.

**Normalized contract mapping:** source and legacy providers are compared at the
same contract boundary. The manifest `expected_outputs` list must include the
following exact roles, and provider validation rejects missing or unexpected
roles:

- metadata fields `name = "musl-seed-toolchain"`, `target =
  "x86_64-linux-musl"`, and `dynamic_linker = "ld-musl-x86_64.so.1"`.
- compiler/tool roles in `bin/`: `x86_64-linux-musl-gcc`,
  `x86_64-linux-musl-g++`, `x86_64-linux-musl-c++`,
  `x86_64-linux-musl-cpp`, `x86_64-linux-musl-gcc-ar`,
  `x86_64-linux-musl-gcc-nm`, `x86_64-linux-musl-gcc-ranlib`,
  `x86_64-linux-musl-ar`, `x86_64-linux-musl-as`,
  `x86_64-linux-musl-ld`, `x86_64-linux-musl-nm`,
  `x86_64-linux-musl-objcopy`, `x86_64-linux-musl-objdump`,
  `x86_64-linux-musl-ranlib`, `x86_64-linux-musl-readelf`,
  `x86_64-linux-musl-size`, `x86_64-linux-musl-strings`, and
  `x86_64-linux-musl-strip`.
- header role `<target>/include` with Linux UAPI and musl headers.
- library roles `<target>/lib/libgcc_s.so.1`, `<target>/lib/libc.so`, and the
  C++ runtime files required by the current seed contract.
- metadata role `share/crunch-bootstrap/provider.json` with
  `reduction.retained_tools`, `reduction.dropped_components`, and provider
  notes.

Later bootstrap derivations may import only `bootstrap/seed.ncl` fields and the
normalized output path. Tests must fail if a later derivation reads a provider-
specific raw layout such as the legacy raw tarball path.

**Rationale:** this preserves the existing bootstrap derivations and confines
trust-root churn to the provider layer.

**Alternative:** rewrite later bootstrap stages for a new provider layout.

**Why not:** that couples every stage to source-root details and makes review
larger than necessary.

### 3. Use evidence-gated claim promotion

**Choice:** docs may say full-source root evidence exists only after manifest
validation, provider build, and self-build proof all succeed.

**Proof binding:** `crunch self-build --source-root <manifest> --no-substitute`
first validates the manifest, builds/materializes the source-root provider,
computes the manifest digest and provider output digest, stages source with the
source-root-generated `bootstrap/seed.ncl`, then runs the existing self-build
steps. Invalid manifest, provider validation failure, legacy musl.cc dependency
trace, missing provider digest, or `--source-root` combined with an explicit
legacy provider selection fails before the proof can claim full-source evidence.
Proof metadata records `provider_kind`, `source_root_manifest_digest`,
`provider_output_digest`, and `proof_bundle_digest`. To avoid self-reference,
`proof_bundle_digest` is the BLAKE3 digest of the canonical proof-bundle
manifest before adding the reporting field that prints the digest; human summary
text may echo the digest but is not part of the digest input. `provider_kind =
"source-root"` is required before any full-source claim can be emitted. Legacy
`--fetch` and default seed-assisted runs write `provider_kind =
"legacy-fetched"` and remain blocked from satisfying the full-source claim even
when the rest of the self-build proof succeeds. The summary must name any
missing digest or legacy provider kind as the blocked-claim reason.

**Rationale:** the repo has been careful not to over-claim. The new claim needs
the same discipline.

## Implementation Sketch

1. Add a typed manifest parser/validator in a pure core module.
2. Add `crunch bootstrap --source-root <manifest>` for the source-built provider
   and make `--fetch --source-root <manifest>` fail before provider work starts.
3. Add provider-building code that materializes the source-built provider and
   emits provider metadata matching the current schema.
4. Add tests that compare contract fields from legacy and source-built
   providers without requiring byte-identical provider internals.
5. Extend self-build proof reports with provider kind, manifest digest,
   provider output digest, and proof bundle digest; `self-build --source-root`
   must bind proof evidence to the selected provider.
6. Update README and bootstrap inventory only after proof evidence exists.

## Risks / Trade-offs

**Source root may still require a tiny seed.** The manifest can model that, but
the docs must be precise about what remains trusted.

**Provider build may be slow.** Keep focused provider tests separate from the
full proof, and use the full proof only for acceptance evidence.

**Digest algorithm interoperability.** crunch-owned digests default to BLAKE3;
interop-only hashes need explicit reasons.

## Validation Plan

- Manifest positive and negative tests.
- Provider contract tests for required fields and retained tools.
- Full self-build proof with source-built provider.
- Docs/proof-report negative checks that blocked claims name missing evidence,
  legacy provider reports stay seed-assisted, and successful source-root proof
  is the only path that promotes the full-source claim.
- `openspec validate full-source-bootstrap-root --strict`.
