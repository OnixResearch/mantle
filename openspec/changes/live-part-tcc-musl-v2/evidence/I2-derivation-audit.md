Task-ID: I2
Covers: bootstrap.part.tcc.musl.v2

# tcc musl v2 derivation audit

Checked file: `bootstrap/tcc-musl-v2.ncl`.

Result: PARTIAL. Dependency order and source pin are present; archive contract remains conditional.

Matches upstream intent:

- Imports first musl-linked tcc and second-pass musl.
- Fetches tcc 0.9.27 with fixed hash.
- Builds once with prior tcc, then self-hosts by rebuilding `tcc` with `./tcc-musl-v2`.
- Installs `bin/tcc` and `bin/tcc-0.9.27-musl-v2`.

Current Crunch deviations / risk:

- Uses x86_64 compile definitions while upstream script examples are i386-oriented.
- Does not explicitly rebuild `libtcc1.a`; it only copies `libtcc1.a` if present.
- Does not assert `lib/tcc/libtcc1.a`, so I3/V3 should tighten or document the output contract.
