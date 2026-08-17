## ADDED Requirements

### Requirement: Source-built Rust provider fixed-point handoff

r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point] Mantle MUST run provider-backed Cargo-free one-shot and fixed-point proof stages with the explicit receipt-bound source-built Rust/native toolchain closure when that closure is supplied.

#### Scenario: compatibility probes use the receipt-bound closure

GIVEN a Cargo-free proof is launched with `--rust-source-provider` and `--toolchain-closure`
WHEN Mantle probes whether the selected Rust compiler accepts proof-required rustc flags
THEN the probe MUST run with the receipt-bound toolchain PATH aliases derived from the explicit closure.
AND the probe MUST NOT discover C compilers, linkers, or helper tools from ambient PATH entries.
AND a failed probe MUST produce a deterministic source-built closure blocker instead of creating a compatibility wrapper outside the closure.

#### Scenario: source-root unwind archive is declared

GIVEN Mantle materializes a native closure from the source-root musl provider layout
WHEN the source-root GCC runtime contains an unwind archive used to satisfy Rust `-lunwind` links
THEN the manifest MUST record that unwind archive as a source-built runtime member with BLAKE3 digest and source/build receipt identity.
AND the receipt-bound C compiler alias MUST expose only that declared archive as `libunwind.a` for Rust linker compatibility.

#### Scenario: fixed-point evidence stays bounded

GIVEN a provider-backed Cargo-free one-shot or fixed-point proof reaches a blocker or succeeds
WHEN Mantle writes the proof summary and evidence
THEN the evidence MUST report whether the explicit closure was enforced, the policy digest used by each completed stage, and the exact blocker if any stage fails.
AND the evidence MUST NOT claim release reproducibility, full Cargo compatibility, or a broader bootstrap proof from provider-backed fixed-point evidence alone.
