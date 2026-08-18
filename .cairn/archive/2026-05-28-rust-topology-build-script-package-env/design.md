# Design

## Boundary

The functional core parses package metadata into deterministic key/value facts. The imperative shell only applies those facts to rustc and build-script subprocess environments through existing env plumbing.

## Approach

1. Extend native manifest package parsing for bounded optional metadata fields.
2. Build a stable `CARGO_PKG_*` env map from manifest data, including version components and empty-string defaults for absent optional fields.
3. Store the env map on native package/unit summaries.
4. Add package env entries to reviewable rustc derivation env so `env!(...)` macros see Cargo-compatible values during direct rustc compilation.
5. Preserve existing bounded build-script child env and pass through `CARGO_PKG_*` entries from derivation env.
6. Verify with positive and negative env tests plus a topology self-probe.

## Risks

- Cargo exposes many env vars. This change is intentionally bounded to package metadata env vars and does not claim full Cargo environment parity.
- Manifest version inheritance already exists; version env must use the resolved package version, not the raw manifest field.
- Build-script runtime env may need more package fields later; this change provides the same package env map for compile-time and runtime to avoid split behavior.
