## Why

The GCC 4.0 pass1 artifact now has a symbol-shaped `libgcc.a`, but member bodies are placeholders. Promoting one simple member to real semantics creates the first correctness step beyond graph completion.

## What Changes

- **Implement**: Implement one low-risk `libgcc.a` member with testable semantics, preferring `_negdi2` or `_muldi3`.
- **Add**: Add an extraction/symbol/semantic smoke that proves the selected member does not remain a placeholder.
- **Keep**: Keep broader `libgcc2` replacement out of scope for this slice.

## Capabilities

### New Capabilities
- `gcc40-libgcc-semantic-member`: Promote first GCC 4.0 libgcc member to real semantics.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, GCC artifact smoke/evidence commands.
- **APIs**: No public API change unless implementation tasks discover a necessary narrow seam.
- **Dependencies**: No new default dependency expected.
- **Testing**: Each task records the smallest relevant command or evidence artifact.
