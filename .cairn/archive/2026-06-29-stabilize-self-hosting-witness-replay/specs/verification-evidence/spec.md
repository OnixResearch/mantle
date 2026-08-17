# Verification Evidence Specification Delta

## ADDED Requirements

### Requirement: Self-hosting witness replay normalizes bootstrap and generated path identity [r[verification_evidence.self_hosting_witness_replay_path_normalization]]

Mantle release witness rebuilds that run the self-hosting proof workflow MUST normalize bootstrap tool paths, Cargo target paths, build-script output paths, and staged source paths before those paths can affect Cargo fingerprints, rustc diagnostics, generated-code source spans, or final release binary bytes. The witness MUST still compare every rebuilt published output by exact BLAKE3 digest and MUST fail closed before signing when normalization is incomplete or rebuilt bytes differ.

#### Scenario: stable bootstrap aliases drive Cargo fingerprints

GIVEN a self-hosting proof uses equivalent admitted bootstrap tool objects whose content-addressed paths differ between publisher and witness machines
WHEN the generated self-build script configures Cargo, linker flags, and compile-time helper environment
THEN it MUST use stable in-sandbox aliases for bootstrap tool paths in Cargo-visible values
AND the proof manifest MUST still record the real tool object refs and real content-addressed paths used behind those aliases.

#### Scenario: generated Cargo paths are remapped before rustc embeds them

GIVEN Cargo invokes rustc for a crate whose build script produced generated Rust under `OUT_DIR`
WHEN the self-hosting proof runs in deterministic witness mode
THEN rustc arguments MUST remap `CARGO_TARGET_DIR`, crate `OUT_DIR`, staged source roots, and stable bootstrap aliases to deterministic logical prefixes
AND final release binaries MUST NOT contain host-specific generated paths such as `/tmp/cargo-target/.../build/<crate>-<host-dependent-hash>/out`.

#### Scenario: build scripts keep real filesystem paths

GIVEN deterministic witness mode remaps path identity at the rustc boundary
WHEN a build script reads checked-in package files, invokes helper tools, or writes generated outputs
THEN the build script MUST still run with real package roots and real `OUT_DIR` values for filesystem I/O
AND path normalization MUST NOT replace paths that the build script must open or create.

#### Scenario: exact witness acceptance is preserved

GIVEN a witness rebuild produces a self-hosting proof bundle whose stage artifact differs from a published release output
WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
THEN the command MUST exit non-zero before writing witness sidecars even if the binary matches after stripping symbols or other post-processing
AND the audit metadata MUST report the expected digest, rebuilt digest, self-hosting fixed-point status, provider proof status when present, and any detected bootstrap/path-normalization divergence.

#### Scenario: separate-machine replay evidence stays bounded

GIVEN Aspen or another separate machine successfully rebuilds all published release outputs after self-hosting normalization
WHEN the result is summarized as witness evidence
THEN the summary MUST record witness identity, host class, release id, output digests, provider proof status, self-hosting stage digests, and final verification status
AND the claim MUST remain limited to the exact release artifact set and proof inputs unless separate evidence proves broader compiler or bootstrap correctness.
