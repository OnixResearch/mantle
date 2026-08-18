## Context

The current sidecar was intentionally minimal so build reports could identify sandboxed offline Cargo outputs without scraping logs. As the offline build lane becomes more useful, operators need evidence that the exact lockfile, package source, vendored material, and toolchain inputs used by Cargo are the ones they intended. Reports also need to make malformed evidence visible instead of quietly omitting it.

## Decisions

### 1. Evidence v2 binds identities, not just paths

**Choice:** The new sidecar records BLAKE3 digests for Mantle-owned source inputs and Cargo.lock, and records Cargo SHA-256 checksum metadata only where Cargo interoperability requires it. Toolchain entries include path plus digest or admitted source-state identity when available.

**Rationale:** Paths alone are not durable evidence. BLAKE3 gives Mantle a stable content identity while Cargo checksum metadata remains useful for vendored dependency validation.

### 2. Network status is explicit

**Choice:** The sidecar records the selected offline/network policy and the observed network-policy result for the Cargo build action. The expected successful offline Cargo evidence path records offline mode and no undeclared network allowance.

**Rationale:** `cargo build --offline` is necessary but not enough. Mantle's sandbox/network policy must be visible in the evidence boundary.

### 3. Malformed evidence becomes a diagnostic

**Choice:** Build reports continue to ignore unrelated outputs with no sidecar, but if an output contains a sidecar path with malformed JSON, wrong schema, wrong claim class, or missing mandatory v2 fields, the JSON report records a deterministic diagnostic rather than silently dropping it.

**Rationale:** A malformed sidecar on an expected offline Cargo output is evidence corruption. Operators should see it without parsing the output tree manually.

### 4. Legacy sidecars are accepted but labeled

**Choice:** Existing v1 sidecars can still be surfaced as legacy offline Cargo evidence, but they do not satisfy digest-bound evidence requirements and must carry the same non-claims.

**Rationale:** Compatibility avoids breaking old local outputs while making it clear that stronger evidence requires new builds.

### 5. Claim boundaries stay unchanged

**Choice:** Evidence v2 remains `cargo-inside-mantle-sandbox`. It can prove declared offline Cargo action identity and input binding, not Cargo-free execution or semantic correctness.

**Rationale:** Stronger sidecar fields should not create stronger claims than the execution lane actually proves.

## Risks / Trade-offs

- Computing full tree digests in the shell must use bounded traversal and deterministic ordering to avoid large or unstable evidence generation.
- Some toolchain inputs may be huge; when a full digest is unavailable, the sidecar must record an explicit narrower identity class instead of inventing a hash.
- Legacy compatibility may create two evidence shapes in reports; docs and tests should make the distinction clear.
