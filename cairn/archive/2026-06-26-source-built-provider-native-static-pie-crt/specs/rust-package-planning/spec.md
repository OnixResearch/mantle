## ADDED Requirements

### Requirement: Source-built provider native static-PIE CRT normalization

r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt] Mantle MUST normalize provider-backed Cargo-free native topology links away from unsupported source-root musl static-PIE CRT inputs when the receipt-bound closure supplies only non-PIE static CRT material.

#### Scenario: Receipt-bound alias replaces static-PIE startup object

GIVEN a provider-backed Cargo-free topology unit links through the receipt-bound C compiler alias
WHEN Rust passes `rcrt1.o` or `-static-pie` directly or through a readable linker response file
THEN Mantle MUST pass the manifest-declared private `crt1.o` to the source-root musl C compiler instead of the original `rcrt1.o`.
AND Mantle MUST rewrite readable response files under the alias runtime directory before forwarding them to the source-root C compiler.
AND Mantle MUST append `-no-pie` when `-static-pie` is downgraded to `-static` so source-root GCC defaults cannot keep the final link in PIE mode.
AND Mantle MUST keep the replacement under the alias runtime directory rather than using an ambient sysroot path.

#### Scenario: CRT facts are manifest-bound

GIVEN an explicit source-built toolchain closure manifest is supplied
WHEN Mantle prepares the receipt-bound C compiler alias for native topology execution
THEN Mantle MUST derive the private CRT object from exactly one declared target CRT closure member.
AND Mantle MUST fail closed instead of guessing a CRT path when the declared target CRT member is missing or ambiguous.

#### Scenario: Provider proof frontier is rerun honestly

GIVEN native static-PIE CRT normalization has been addressed
WHEN the provider-backed fixed-point proof is rerun from current code
THEN Mantle MUST record whether fixed-point succeeds or the next deterministic blocker appears.
AND any blocked run MUST retain bounded non-claims instead of reporting provider fixed-point release artifact evidence.
