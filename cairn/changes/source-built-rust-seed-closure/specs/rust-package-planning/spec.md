## ADDED Requirements

### Requirement: Source-built Rust seed closure

r[rust_package_planning.source_built_rust_seed_closure] Mantle MUST NOT treat Rust compiler or Rust sysroot seed exceptions as source-built toolchain closure members unless they are backed by receipt-bound source-built provider metadata.

#### Scenario: Rust provider metadata is complete

GIVEN a source-built Rust compiler/sysroot provider is supplied to a Cargo-free source-built closure proof
WHEN Mantle validates the toolchain closure
THEN the provider MUST identify Rust compiler executables, target standard libraries, host support artifacts, source identities, build receipt identities, executable paths, and BLAKE3 content digests.
AND missing provider metadata, placeholder metadata, digest mismatch, or prebuilt-only provenance MUST fail closed before any source-built closure claim is reported.

#### Scenario: Rust seed exceptions remain non-claims

GIVEN the proof still depends on a prebuilt Rust compiler, prebuilt Rust sysroot, or wrapper around an ambient Rust toolchain
WHEN the proof summary is written
THEN Mantle MUST keep the relevant seed exceptions explicit and MUST continue reporting that the run is not a full source-built compiler/toolchain closure.
AND fixed-point binary equality MUST NOT by itself promote those seed exceptions into source-built closure evidence.
