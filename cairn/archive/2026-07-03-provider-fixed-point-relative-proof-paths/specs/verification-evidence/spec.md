## MODIFIED Requirements

### Requirement: Provider fixed-point replay normalizes local path identity [r[verification_evidence.provider_fixed_point_path_normalization]]

Provider-bound release witness replay MUST compile provider fixed-point stages with deterministic source, execution-output, provider helper, and receipt-bound C compiler toolchain path identity so binary digest mismatches identify source, toolchain, or build output differences instead of publisher/witness scratch path differences.

#### Scenario: deterministic path mode is receipt-visible

GIVEN `mantle release witness-rebuild` runs a provider fixed-point proof for a provider-bound request
WHEN the witness proof invokes native rust-plan execution
THEN the rust-plan receipt MUST record deterministic release path mode
AND rustc arguments MUST include stable remap prefixes for the source root and execution-output root.

#### Scenario: provider helper paths are not baked into release binaries

GIVEN the publisher and witness use equivalent source-built Rust provider closures at different filesystem paths
WHEN provider fixed-point stages compile crates that read compile-time provider helper environment
THEN the compile-time environment MUST use deterministic placeholder identity for provider helper paths
AND the released binary MUST NOT depend on the publisher or witness provider scratch path.

#### Scenario: native C compiler paths are remapped deterministically

GIVEN the selected receipt-bound C compiler route points inside a local source-built toolchain root
WHEN provider fixed-point stages compile native C or assembly inputs through build-script-driven toolchains
THEN build-script child environments MUST add C prefix-map flags for the source root, execution-output root, and selected C compiler toolchain root
AND the C compiler toolchain root remap MUST take precedence over broader source-root remaps in emitted debug/source identity.

#### Scenario: build scripts still access real package roots

GIVEN deterministic release path mode is enabled for provider fixed-point replay
WHEN a build script runs during native topology execution
THEN the build script process MUST still execute from the real package root and write to the real OUT_DIR
AND deterministic path remapping MUST NOT replace filesystem paths that the build script must open or create.

#### Scenario: proof-owned metadata paths are bundle-local

GIVEN a provider fixed-point proof writes `meta.json` or `preflight.json`
WHEN the metadata names proof-owned stage directories, receipts, logs, status files, execution directories, or stage binaries
THEN those paths MUST be relative to the proof bundle root
AND source roots, provider/toolchain provenance paths, and other non-bundle evidence paths MUST remain explicit external paths rather than being rewritten as bundle-local artifacts.
