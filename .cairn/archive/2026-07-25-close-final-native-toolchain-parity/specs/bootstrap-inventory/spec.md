## ADDED Requirements

### Requirement: Final native toolchain parity preserves derivational lineage

r[bootstrap_inventory.final_native_toolchain_parity] Mantle MUST mark `gcc.4.7`, `gcc.10`, and `full-musl-binutils` complete only when current artifacts form a BLAKE3-bound derivational closure from the admitted early compiler boundary through regenerated compiler stages to final libc and binutils outputs without state-pinned overlay, impure transitive, host-tool, or undeclared generated-source substitution.

#### Scenario: GCC 4.7 follows the admitted predecessor

GIVEN the early native GCC boundary is complete
WHEN Mantle generates, builds, and evaluates GCC 4.7
THEN required generated sources and compiler/runtime artifacts MUST be produced by declared source-built generators and the admitted immediate predecessor
AND release-generated substitution, wrong-predecessor output, host compilation, state-pinned overlay input, or incomplete receipts MUST keep `gcc.4.7` blocked.

#### Scenario: GCC 10 follows GCC 4.7

GIVEN GCC 4.7 has complete stage-local evidence
WHEN Mantle generates, builds, and evaluates GCC 10
THEN GCC 10 C/C++ drivers, compiler internals, libgcc, C++ runtime, and required generated artifacts MUST bind the GCC 4.7 receipt and authenticated source records
AND simple compile success MUST NOT hide undeclared host tools, missing runtime members, stale generated files, or predecessor substitution.

#### Scenario: final musl and binutils close the provider

GIVEN GCC 10 has complete stage-local evidence
WHEN Mantle builds final musl and binutils and normalizes the provider
THEN the output MUST bind headers, CRT, static/shared libc, libgcc and C++ runtimes, dynamic interpreter, assembler, linker, archive, ranlib, inspection, strip, relocation, and copied-tree behavior to current artifacts
AND missing surfaces, embedded unavailable paths, malformed-input crashes/timeouts, impure closure members, or stale receipts MUST keep `full-musl-binutils` blocked.

#### Scenario: final parity claims remain bounded

GIVEN all three final rows are complete
WHEN the result is reported
THEN Mantle MUST identify every immediate predecessor, source/generator receipt, output digest, runtime/rejection result, closure scan, and relocation result
AND it MUST NOT claim compiler, libc, binutils, seed, or whole-bootstrap correctness.