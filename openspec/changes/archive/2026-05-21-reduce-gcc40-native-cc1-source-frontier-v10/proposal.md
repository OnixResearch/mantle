# Change: reduce-gcc40-native-cc1-source-frontier-v10

## Summary

Record one bounded GCC 4.0 native `cc1` source-frontier reduction by tightening the existing focused `c-parse.o` diagnostic from generic make-log size/tail evidence to an include-flood signature at the active failure seam.

## Motivation

The current v9 frontier proves the diagnostic reaches the real `c-parse.o` make attempt and records line/byte counts plus a bounded tail. The remaining evidence is still broad: it says the make step failed, but does not name a stable diagnostic signature inside the failure. Capturing an exact include-flood presence marker gives the next source-frontier drain a smaller, inspectable target without claiming native GCC correctness.

## Scope

- Update `bootstrap/diag-gcc40-c-parse-boundary.ncl` to emit compact include-flood presence/count markers from the focused make log.
- Update `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` to schema `mantle-gcc40-native-cc1-source-frontier-reduction-v10`.
- Update fail-closed parity validation/tests and canonical bootstrap spec scenarios.

## Non-goals

- No promotion of `gcc.4.0` beyond `partial`.
- No claim that native GCC 4.0 `cc1`, `c-parse.o`, or full source build works.
- No broad diagnostic matrix expansion that risks the builder argument-size limit.
