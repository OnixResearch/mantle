## Design

### Receipt contract

Bump the existing native `cc1` bounded-slice receipt to a v5 schema instead of adding a parallel file. The v5 receipt's selected slice is `array-index-v5` and it preserves the prior v1 arithmetic, v2 logical, v3 local-variable, and v4 helper-call regressions.

Required selected-slice evidence:

- bounded input contains a local fixed `int` array declaration;
- bounded input writes at least two indexed elements and reads them through indexed expressions;
- transcript records a deterministic marker for the v5 array-index slice;
- transcript digest recomputes from the receipt;
- output digest is BLAKE3-shaped;
- no TinyCC delegation markers appear in selected or regression transcripts;
- derivation contains the selected no-delegation marker and the prior regression no-delegation markers.

### Non-claims

This remains one bounded installed-`cc1` proof input. It does not prove native GCC 4.0 compiler correctness, generator correctness, demangler completeness, full live-bootstrap parity, Guix parity, or StageX parity.

### Archive note

When archiving, inspect `openspec/specs/bootstrap/spec.md` for cumulative-spec drift. Restore the pre-archive cumulative requirement and append only the array-index scenarios if OpenSpec replaces prior scenario text.
