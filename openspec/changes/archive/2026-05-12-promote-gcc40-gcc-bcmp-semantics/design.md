## Context

The GCC 4.0 pass1 archive is intentionally symbol-shaped while selected libgcc members are promoted one at a time. `__gcc_bcmp` is small, deterministic, and already present in the archive, making it a good semantic slice.

## Decisions

### Implement byte-wise `__gcc_bcmp`

Use the standard contract shape: compare `size` bytes and return `0` for equality or a non-zero byte difference for the first mismatch.

### Verify inside the derivation

Compile and link a tiny smoke against the generated member source/object before assembling the archive, checking equal and unequal cases. This stays local to the derivation and does not require a full native `cc1`.

### Keep parity blocking

This is only one member promotion. `gcc.4.0` remains evidence-backed partial and blocks live-bootstrap/Guix until native compiler correctness is proven.
