## Why

GCC 4.0 now reaches a pass1 graph-complete artifact, but many objects and executables are bridged or stubbed. Correctness promotion spans multiple components and needs explicit sequencing before implementation.

## What Changes

- **Define**: Define the ordered path from pass1 bridge artifact to correctness-oriented GCC 4.0 validation.
- **Separate**: Separate libgcc semantics, driver/preprocessor behavior, cc1/generator replacement, and smoke-test evidence into bounded tasks.
- **Preserve**: Preserve the caveat that graph completion is not native/self-hosted GCC correctness.

## Capabilities

### New Capabilities
- `gcc40-correctness-roadmap`: Plan GCC 4.0 correctness promotion beyond graph completion.

## Impact

- **Files**: OpenSpec bootstrap specs/tasks; later `bootstrap/gcc-4.0.ncl` changes are out of scope until the plan is accepted.
- **APIs**: No public API change unless implementation tasks discover a necessary narrow seam.
- **Dependencies**: No new default dependency expected.
- **Testing**: Each task records the smallest relevant command or evidence artifact.
