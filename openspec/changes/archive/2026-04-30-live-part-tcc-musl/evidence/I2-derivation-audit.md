Task-ID: I2
Covers: bootstrap.part.tcc.musl

# tcc 0.9.27 musl derivation audit

Checked file: `bootstrap/tcc-musl.ncl`.

Result: PARTIAL. The derivation pins source and builds against first musl, but upstream self-host loop and `libtcc1.a` contract need stronger proof.

Matches upstream intent:

- Imports bridge compiler `tcc-musl-prep.ncl` and first musl `musl-1.1.24-tcc.ncl`.
- Fetches tcc 0.9.27 with fixed hash.
- Configures `CONFIG_TCC_CRTPREFIX`, `CONFIG_TCC_LIBPATHS`, and include paths against the musl output.
- Installs musl-linked compiler names under `$out/bin`.

Current Crunch deviations / risk:

- Uses x86_64 target define while upstream script is i386-oriented.
- Does not mirror upstream two-pass loop (`tcc-0.9.26` then `./tcc-musl`) in the same way; it uses the bridge compiler once.
- Only copies `libtcc1.a` if present, so downstream archive contract is conditional.
- Does not explicitly rebuild `libtcc1.a` during this stage.
