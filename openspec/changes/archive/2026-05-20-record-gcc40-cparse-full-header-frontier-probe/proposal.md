# Proposal: record GCC 4.0 c-parse full generated-header frontier v6

## Why

The v5 source-frontier receipt narrowed the diagnostic seam to representative generated-header prefix/balance probes. The same focused diagnostic matrix now has stronger bounded evidence: later `machmode.h` prefixes, wider `tree.h` prefixes through 220 lines, and builtin enum probes pass, while the real `c-parse.o` make target still fails at the full c-parse source-build boundary.

## What changes

- Advance `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` source-frontier metadata from v5 to v6.
- Record the full generated-header sweep boundary and the still-failing real `c-parse.o` make attempt.
- Update fail-closed parity validation, fixtures, and canonical bootstrap spec wording.

## Non-goals

- Do not promote `gcc.4.0` to complete.
- Do not claim native GCC 4.0 compiler correctness or a full native `cc1` source-build proof.
- Do not expand installed-`cc1` semantic micro-slices.
