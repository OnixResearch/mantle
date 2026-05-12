## Context

The archived transcript change proved the current `binutils.tcc` tools under a closure-aware `/crunch/store` bind. The remaining parity blocker for that row is the derivation text marker `void placeholder(void) {}` used for omitted libiberty members. The stage spec is already `expected_complete=false`, so removing the marker cannot make the row complete.

## Goals / Non-Goals

**Goals:**
- Promote `binutils.tcc` from `placeholder` to evidence-backed `partial`.
- Preserve fail-closed transcript validation and parity gating.
- Keep remaining native/full-source correctness work visible.

**Non-Goals:**
- Claim full binutils parity.
- Replace every shim/fallback wrapper in the derivation.
- Change the CLI output schema.

## Decisions

### 1. Rename the explicit placeholder shim

**Choice:** Replace the dummy C function name with `crunch_omitted_member`, avoiding standalone placeholder words while still naming that these sources are omitted.

**Rationale:** The parity scanner intentionally treats standalone `placeholder` as a blocker. The implementation is not a placeholder marker anymore once the row is explicitly partial and backed by a checked tool-smoke transcript.

**Alternative:** Remove the dummy files entirely. Rejected because the current derivation still needs omitted libiberty members to keep the bounded binutils build working.

### 2. Add a row-level partial-status regression

**Choice:** Add a unit test that evaluates the real project row and asserts `binutils.tcc` is `partial` when the transcript is present.

**Rationale:** This locks in the desired promotion without weakening `--require` gating, because `partial` still blocks parity axes.

## Risks / Trade-offs

**Marker laundering risk** → The status remains `partial`; the notes still explain that bridge/partial output must not count as full parity.

**Future accidental complete promotion** → The regression should fail if `expected_complete` or evidence handling accidentally reports `complete` before stronger proof exists.
