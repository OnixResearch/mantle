# Design

The native boundary receipt is the compact control-plane artifact that names the intentional GCC 4.0 native frontier after bounded semantic slices. This change keeps that artifact aligned with the parity row wording by requiring an explicit `generator-bounded-outputs` frontier blocker.

Validation should remain narrow:

- require the blocker id in `native_frontier.blockers`;
- require its source-bound derivation marker to be present in `bootstrap/gcc-4.0.ncl`;
- reject omission through a focused unit regression;
- avoid changing generator outputs or claiming more than bounded frontier evidence.
