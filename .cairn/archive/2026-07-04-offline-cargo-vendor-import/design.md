## Context

The current Cargo import scaffold is intentionally conservative: it accepts bounded path dependency workspaces and blocks registry/git dependencies as unsupported source material. That was safe for the initial import lane, but it creates needless friction because the offline Cargo helper already supports an explicit vendored dependency input.

The important boundary is not “no registry dependencies”; it is “no undeclared dependency material.” A Cargo vendor tree that is lock-bound and checksum-verified is declared dependency material and should be admitted as a source input. Ambient Cargo home directories, network registries, and target directories remain undeclared and must stay blocked.

## Decisions

### 1. Vendored source detection is a shell concern

**Choice:** The CLI shell reads `.cargo/config.toml` / `.cargo/config` source replacement facts and inspects the declared local directory. The pure import planner receives normalized `VendorSourceFacts` containing vendor root identity, package entries, checksums, and blockers.

**Rationale:** Filesystem reads and TOML parsing belong in the imperative shell; deciding whether facts are sufficient for a scaffold is pure, deterministic planning logic.

### 2. Cargo checksum metadata is the admission boundary

**Choice:** Vendored registry/git package entries are accepted only when the package is selected by `Cargo.lock`, the directory name/version maps unambiguously, and `.cargo-checksum.json` metadata proves the checked files match the expected Cargo checksums. Mantle-owned source fingerprints use BLAKE3 in reports, while Cargo's SHA-256 checksum files are retained for Cargo interoperability.

**Rationale:** Cargo itself uses the checksum metadata to protect vendored sources. Mantle should not invent weaker acceptance rules or conflate Cargo-compatible SHA-256 metadata with Mantle-owned BLAKE3 report identities.

### 3. Generated Nickel keeps vendor material explicit

**Choice:** When vendor facts are accepted, `mantle import cargo --apply` generates a vendor placeholder in `.mantle/inputs.ncl` and passes `vendor_src` / `vendor_name` to `mantle.offlineCargoPackage`.

**Rationale:** The generated project should make every offline build input reviewable. The package builder can then copy the declared vendor tree into isolated Cargo home/config state without consulting ambient caches.

### 4. Unsupported source replacement remains a blocker

**Choice:** Non-directory source replacement, multiple replacement chains, missing vendor directories, malformed checksum metadata, lockfile/package ambiguity, and unselected dependency material produce deterministic blockers. The planner does not silently downgrade to path-only import or partial generated files.

**Rationale:** A generated project that omits required dependency material would look buildable but fail later or reach for network. The import scaffold must stay honest and fail before mutation.

### 5. Import support is not network vendoring

**Choice:** Reports and docs describe this as accepting pre-existing vendored source material. They explicitly exclude network vendoring, full Cargo compatibility, compiler correctness, and Cargo-free execution.

**Rationale:** The improvement makes the offline Cargo lane practical; it does not prove the broader native Rust planner or the Cargo ecosystem.

## Risks / Trade-offs

- Cargo vendor directory naming and lockfile package identity can be ambiguous across duplicate names/versions; the first implementation should fail closed when identity is not unique.
- Cargo git dependencies may appear in vendor trees with registry-like checksum metadata but different source identities; tests must cover both accepted and rejected cases.
- Parsing only `.cargo/config.toml` initially may miss legacy `.cargo/config`; if both are supported, precedence must be deterministic.
- The generated placeholder still requires the operator to materialize the vendor input; docs should point to source-bundle import/export or explicit derivations rather than ambient paths.
