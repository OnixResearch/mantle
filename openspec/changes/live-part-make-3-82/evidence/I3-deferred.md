Task-ID: I3
Covers: bootstrap.part.make.3.82

# make 3.82 implementation deferred

Result: DEFERRED to OpenSpec change `repair-make-tcc-amd64-varargs`.

During implementation, `bootstrap/make-tcc.ncl` exposed an amd64-specific blocker instead of a local source-pin/output-contract cleanup:

- Building with the archived Mes-linked `tinycc 0.9.27` hangs at the first C compile (`tcc -c getopt.c`). This matches the tinycc 0.9.27 handoff evidence: that compiler is version-capable, not compile-boundary capable on amd64.
- Switching the make pass to compile-capable `tinycc 0.9.26` lets all make objects compile, but the resulting GNU make binary segfaults on a simple Makefile.
- Adding prototype-bearing config defines removes most pointer-truncation warnings, but the binary still crashes inside make's varargs-heavy `concat(...)` path, consistent with Mes/tcc amd64 varargs limitations already worked around in `bootstrap/tinycc.ncl`.

The failed transcripts are preserved as diagnostic evidence only:

- `evidence/V2-build-full.log` shows the successful wrapper build experiment and derivation logs from the local debug attempt; it is not PASS evidence for this parent part.
- `evidence/V3-smoke-full.log` shows `make --version` works through the wrapper but a simple Makefile still segfaults; it is not PASS evidence for this parent part.

Next action: `repair-make-tcc-amd64-varargs` must decide whether to patch GNU make's varargs/string construction paths, carry a smaller bootstrap make, or change the predecessor compiler boundary. This is larger than the part-local drain budget, so this parent part defers rather than hiding the blocker.
