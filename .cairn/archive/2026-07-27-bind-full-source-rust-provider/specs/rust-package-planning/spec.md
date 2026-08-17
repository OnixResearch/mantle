## ADDED Requirements

### Requirement: Full-source Rust provider closure binding

r[rust_package_planning.full_source_rust_provider_binding] Mantle MUST classify a Rust compiler/sysroot closure as full-source-bound only when every claimed compiler, sysroot, native tool, and runtime member is linked to the admitted full-source native provider or to the Rust stages built from that provider, with no seed exceptions inside the claimed closure.

#### Scenario: complete closure is admitted

GIVEN a Rust provider was constructed from the admitted native provider
WHEN Mantle validates its native toolchain closure
THEN the closure MUST bind exact paths, roles, platforms, source/build receipt identities, and BLAKE3 digests for Rust compiler artifacts, host and target rustlibs, C/C++ tools, linker/binutils tools, CRT, libc, libgcc, unwind support, and required native helpers
AND validation MUST report zero seed exceptions before emitting `full-source-bound` status.

#### Scenario: closure substitution is rejected

GIVEN a closure member is missing, unreadable, digest-mismatched, platform-incompatible, backed by an unbound wrapper, or supplied by ambient PATH or runtime search
WHEN Mantle validates or executes the closure
THEN it MUST emit a deterministic member-specific blocker before Rust planning, compatibility probing, or compilation
AND it MUST preserve the narrower non-claim instead of downgrading enforcement.

#### Scenario: proof stages consume one closure identity

GIVEN a later Cargo-free or self-build proof selects the full-source-bound Rust provider
WHEN any compatibility probe, build script, proc macro, rustc unit, linker, or smoke program executes
THEN execution MUST use the receipt-bound closure and record its policy digest
AND an out-of-closure executable or runtime resolution MUST fail the proof.