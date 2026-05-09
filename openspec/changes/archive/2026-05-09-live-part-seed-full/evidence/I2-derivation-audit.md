# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.seed.full

Audited `bootstrap/seed-full.ncl`:

- inputs: `gcc-10.ncl`, `musl-full.ncl`, and `binutils-full.ncl`
- normalized target: `x86_64-linux-musl`
- expected output contract: target-prefixed GCC drivers, binutils, musl headers/libs/startup objects, and provider metadata
- important boundary: this change records normalization-contract hardening only; it does not promote the seed as trusted full-source proof while predecessor runtime proofs remain gated
