# Design: Rust topology output reuse

## Approach

The reuse rail stays per-unit and local to the execution output root:

1. After a successful `execute_rust_unit` run, write a self-reference-safe execution receipt file into that unit's output directory.
2. Before removing or rebuilding an existing output directory, read the prior receipt if present.
3. Digest the currently declared dependency and host artifacts, query the current rustc toolchain identity, and digest existing output artifacts excluding the receipt file itself.
4. Reuse only when the prior receipt matches the current unit identity, source digest, rustc args digest, declared outputs, toolchain identity, dependency/host digests, and current output digests.
5. Return a success receipt with `rebuild_reason = reused-explicit-unit-output` for a valid reuse.
6. Return a structured blocker before invoking rustc when the prior receipt exists but is stale, malformed, or names output artifacts that are now missing/unreadable.
7. Preserve existing rebuild behavior for an output directory with no prior Mantle receipt.

## Receipt material

The existing `RustUnitExecutionReceipt` remains the review surface. Reuse changes only `rebuild_reason`; the receipt still binds source digest, toolchain identity, rustc args digest, dependency and host artifact digests, declared outputs, output artifact BLAKE3 digests, and a stable receipt hash.

## Boundaries

This is not a general cache or substitution feature. Reuse is local to one explicit execution output root and one already-supported unit topology rail.
