# Promote GCC 4.0 native genrecog bounded output slice

## Why

GCC 4.0 remains `partial` because native generator correctness is still represented by a small set of bounded output slices plus explicit frontier bookkeeping. After `genattrtab`, `genoutput`, and `genemit`, the next useful generator seam is `genrecog`, which still emits an empty-recognition boundary. Converting it into checked bounded output evidence reduces the generator frontier without claiming full native generator correctness.

## What Changes

- Add a bounded `genrecog` output contract to the GCC 4.0 native generator receipt.
- Update parity validation and fixtures so missing/stale `genrecog` evidence fails closed.
- Bind the source marker to `bootstrap/gcc-4.0.ncl` and keep `gcc.4.0` evidence-backed `partial`.
- Do not claim full native generator, native compiler, live-bootstrap, Guix, or StageX correctness.

## Impact

- Reduces the generator frontier by one bounded output slice.
- Strengthens audit evidence without promoting GCC 4.0 beyond partial.
- Requires focused parity/CLI/OpenSpec/source-pin/blocker verification before archive.
