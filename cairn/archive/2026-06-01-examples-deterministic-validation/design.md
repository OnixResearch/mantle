## Context

Examples exercise public behavior across Nickel evaluation, glue conversion, build scheduling, fixed-output fetchers, output export, and project selectors. A single all-or-nothing build suite would be slow and flaky, so validation needs a catalog-driven tier model.

## Approach

1. Reuse the examples catalog as the matrix input. Each example declares one or more rails: evaluate, convert, fast-build, execute-output, negative-diagnostic, heavy-manual, or benchmark-compile.
2. Keep pure validation helpers separate from command execution. The core helper should transform catalog entries into expected test cases and output assertions; the shell tests run `mantle`/`crunch` commands and inspect filesystem outputs.
3. Expand evaluation coverage to every cataloged Nickel derivation/project that does not require generated seed material or a documented external capability.
4. Expand build smoke coverage to local/offline examples that can run in a temp store/state and complete quickly.
5. Execute produced scripts or binaries when the output contract is part of the example. For multi-output examples, assert each named output path exists and contains the promised files.
6. Keep heavyweight examples ignored by default but make their commands explicit and evidence-friendly.

## Risks

- Running real bootstrap or network examples in ordinary tests would make CI unreliable.
- Seed-dependent examples can appear broken on a fresh checkout unless the skip reason is explicit.
- Output execution assertions must avoid relying on host `/nix/store` writeability.

## Validation

- Positive tests cover evaluation, conversion, build smoke, and output execution for fast examples.
- Negative tests cover malformed Nickel, intentionally failing builder, missing seed, bad project selector, and expected fixed-output mismatch behavior.
- Heavy examples remain manually runnable with documented ignored tests and commands.
