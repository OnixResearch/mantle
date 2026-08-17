# Build Correctness Specification Delta

## ADDED Requirements

### Requirement: Bootstrap toolchain outputs are deterministic [r[build_correctness.bootstrap_toolchain_determinism]]

Mantle bootstrap derivations that participate in self-hosting or release-witness correctness claims MUST produce deterministic content-addressed outputs for equivalent declared inputs, build scripts, sandbox policy, and bootstrap seed material. Timestamp generation, archive member metadata, locale-sensitive ordering, umask, temporary paths, hostnames, user names, and host tool discovery order MUST either be fixed to deterministic values or excluded from the admitted output identity.

#### Scenario: equivalent GCC bootstrap builds converge

GIVEN two fresh Mantle stores build `bootstrap/gcc.ncl` from equivalent declared seed, source, binutils, musl, make, dash, GMP, MPFR, and MPC inputs
WHEN both builds complete under the same declared sandbox and network policy
THEN the resulting GCC output object digest and content-addressed store path MUST match
AND the build report MUST record the deterministic environment policy used for time, locale, umask, archive behavior, and install metadata.

#### Scenario: nondeterministic bootstrap output blocks strong self-hosting claims

GIVEN two bootstrap toolchain builds from equivalent declared inputs produce different content-addressed output paths or object digests
WHEN Mantle evaluates a self-hosting or release-witness correctness claim that depends on that toolchain
THEN Mantle MUST report the first divergent bootstrap output with deterministic diagnostics
AND it MUST NOT claim cross-machine self-hosting or release-witness reproducibility for downstream binaries built with the divergent toolchain.

#### Scenario: bootstrap path aliases do not weaken tool identity

GIVEN a self-build script maps admitted bootstrap outputs to stable in-sandbox aliases for compiler, linker, archiver, Rust, busybox, or bwrap paths
WHEN Mantle records the build receipt or proof manifest
THEN the receipt MUST bind the real tool object refs and real content-addressed output paths separately from the stable execution aliases
AND stable aliases MUST NOT be accepted as a substitute for matching tool object refs.
