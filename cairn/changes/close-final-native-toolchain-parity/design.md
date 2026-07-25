## Context

GCC 4.7 and GCC 10 outputs have passed bounded C/C++ behavior, and the final GCC 10/musl/binutils provider has passed runtime admission. Their retained evidence nevertheless classifies them as non-parity because release-generated source files, state-pinned inputs, impure transitive closure, or insufficient row-specific receipts remain. The early GCC 4.0 completion is the required predecessor authority for repairing this without host substitution.

## Decisions

### Decision: rebuild each compiler from its immediate admitted predecessor

**Choice:** GCC 4.7 must be generated and built by the completed GCC 4.0 boundary; GCC 10 must be generated and built by the completed GCC 4.7 boundary; final musl and binutils must be built by that GCC 10 output.

**Rationale:** Flattening all tools into one broad provider input would hide which predecessor generated and compiled each artifact.

### Decision: regenerate source-controlled generated artifacts

**Choice:** Parser, scanner, configure, opcode, table, and other generated artifacts required by each compiler stage must be produced by declared source-built generators when the upstream bootstrap lineage requires regeneration. Release tarball generated files may remain only when explicitly classified as source artifacts outside the stronger row claim.

**Rationale:** The parity gap specifically includes release-generated and state-pinned shortcuts.

### Decision: keep receipts stage-local and closure-composable

**Choice:** Each row emits a receipt over immediate predecessor receipts, source records, generation outputs, compiler outputs, runtime tests, and forbidden-input scans. The final normalized provider composes those receipts without replacing them.

**Rationale:** Composition preserves causality while allowing failures to name one stage.

### Decision: test relocation and rejection as first-class behavior

**Choice:** Final compiler/runtime/binutils evidence includes copied-tree execution, dynamic interpreter checks, static and shared C/C++ programs, exceptions/RTTI/allocation, archive/object/relocation operations, and malformed-input rejection with bounded execution.

**Rationale:** In-place version and trivial-compile smokes miss embedded store paths and error-path crashes.

## Risks / Trade-offs

- Regeneration tools may expose dormant bootstrap incompatibilities and require narrow predecessor repairs.
- Full compiler stages are expensive; each stage must preserve reusable authenticated outputs and exact failure logs.
- Passing the bounded matrix does not prove compiler or runtime soundness.