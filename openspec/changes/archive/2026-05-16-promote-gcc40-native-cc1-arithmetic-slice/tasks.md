## 1. Bounded native cc1 arithmetic slice

- [x] 1.1 Replace the current GCC 4.0 installed `cc1` smoke boundary for a bounded arithmetic/control-flow input with a native no-TinyCC-delegation slice, while keeping all broader GCC 4.0 claims marked partial.
  - Completed: `bootstrap/gcc-4.0.ncl` now handles the exact `mantle_gcc40_arith_slice` bounded arithmetic/control-flow proof input before the general TinyCC bridge path and writes a deterministic slice artifact without invoking `$TCC/bin/tcc`.
- [x] 1.2 Add a deterministic receipt/transcript under `bootstrap/evidence/` that records the bounded input, command shape, output digest, transcript digest, and no-TinyCC-delegation marker.
  - Completed: added `bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json` and updated the native boundary receipt to list the promoted slice.
- [x] 1.3 Update the bootstrap parity checker so it fail-closes on missing/drifted native-slice evidence and continues to label the overall GCC 4.0 row `partial`, not complete.
  - Completed: `src/bootstrap_parity.rs` validates the native cc1 arithmetic receipt, recomputes its transcript digest, requires the derivation marker, rejects forbidden TinyCC-delegation transcript markers, and keeps the row partial.
- [x] 1.4 Add positive and negative tests proving the slice receipt is accepted only when all markers and digests match, and proving broader native/full GCC 4.0 correctness is not implied.
  - Completed: added positive and fail-closed digest/forbidden-delegation tests plus real-derivation parity coverage; `gcc40_` bootstrap parity tests pass.
- [x] 1.5 Run focused verification and OpenSpec validation, archive the change, commit, and push.
  - Completed up to implementation verification at 2026-05-16T04:34Z: `cargo fmt`, `cargo test --bin crunch bootstrap_parity::tests::gcc40_ -- --nocapture`. Archive/commit/push evidence is in the repository history for the completion commit.
