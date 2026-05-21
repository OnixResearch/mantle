# Promote GCC 4.0 native genemit bounded output slice

## Why

GCC 4.0 remains `partial` because native generator correctness is only represented by bounded `genattrtab` and `genoutput` output slices plus explicit frontier bookkeeping. The next useful bounded slice is to convert one remaining generator boundary, `genemit`, from an empty-source placeholder into checked deterministic native-generator output evidence.

## What Changes

- Add a bounded `genemit` output contract to the GCC 4.0 native generator receipt.
- Update parity validation and fixtures so missing/stale `genemit` evidence fails closed.
- Bind the source marker to `bootstrap/gcc-4.0.ncl` and keep `gcc.4.0` evidence-backed `partial`.
- Do not claim full native generator, native compiler, live-bootstrap, Guix, or StageX correctness.

## Impact

- Reduces the generator frontier by one bounded output slice.
- Strengthens audit evidence without promoting GCC 4.0 beyond partial.
- Requires focused parity/CLI/OpenSpec/source-pin/blocker verification before archive.
