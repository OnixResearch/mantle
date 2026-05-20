## Design

### Receipt contract

Bump the existing native `cc1` bounded-slice receipt to a v4 schema instead of adding a parallel file. The v4 receipt's selected slice is `function-call-v4` and it preserves the prior v1 arithmetic, v2 logical, and v3 local-variable regressions.

Required selected-slice evidence:

- bounded input contains a helper function definition and a caller function that invokes it;
- transcript records a deterministic marker for the v4 helper-call slice;
- transcript digest recomputes from the receipt;
- output digest is BLAKE3-shaped;
- no TinyCC delegation markers appear in selected or regression transcripts;
- derivation contains the selected no-delegation marker and the prior regression no-delegation markers.

### Non-claims

This remains one bounded installed-`cc1` proof input. It does not prove native GCC 4.0 compiler correctness, generator correctness, demangler completeness, full live-bootstrap parity, Guix parity, or StageX parity.

### Archive note

When archiving, inspect `openspec/specs/bootstrap/spec.md` for cumulative-spec drift. Restore the pre-archive cumulative requirement and append only the helper-call scenarios if OpenSpec replaces prior scenario text.
