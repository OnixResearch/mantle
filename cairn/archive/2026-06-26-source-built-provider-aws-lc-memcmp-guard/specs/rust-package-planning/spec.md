## ADDED Requirements

### Requirement: Source-built provider AWS-LC memcmp guard handling

r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard] Mantle MUST handle AWS-LC's GCC PR95189 `memcmp` compiler guard honestly when running provider-backed Cargo-free topology execution under a source-built native closure.

#### Scenario: Guard failure is a deterministic blocker

GIVEN provider-backed Cargo-free topology execution runs an AWS-LC build script with a receipt-bound source-built C compiler
WHEN AWS-LC's `memcmp_invalid_stripped_check` reports the selected compiler is affected by GCC PR95189
THEN Mantle MUST report a deterministic compiler-guard blocker naming `aws-lc-sys`, the selected compiler identity, and the guard diagnostic.
AND Mantle MUST NOT report provider fixed-point success or source-built release artifact evidence from that blocked run.

#### Scenario: Safe compiler route is receipt-bound

GIVEN a source-built native closure provides more than one C compiler route
WHEN Mantle selects a compiler for AWS-LC C build-script execution
THEN the selected compiler MUST be declared in the closure manifest with role, source identity, build receipt identity, execution path, and BLAKE3 digest.
AND successful AWS-LC guard handling MUST record the selected compiler route in the unit or proof receipt.

#### Scenario: Guard bypasses are forbidden

GIVEN AWS-LC guard handling is required for a provider-backed proof
WHEN Mantle prepares build-script environment or compiler selection
THEN it MUST NOT bypass the guard by spoofing `HOST`/`TARGET`, forwarding undeclared ambient `CC`, forwarding arbitrary `CFLAGS`, suppressing the guard, or falling back to an undeclared host compiler.
AND any unavailable safe route MUST leave the proof blocked with bounded non-claims instead of weakening the source-built closure claim.
