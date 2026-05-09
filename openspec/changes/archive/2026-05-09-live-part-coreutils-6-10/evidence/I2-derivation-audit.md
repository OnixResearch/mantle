# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.coreutils.6.10

- Audited `bootstrap/coreutils-6.10-musl.ncl` source pin, declared TinyCC/musl predecessor inputs, manual config, library object compilation, utility compilation, install layout, and smoke checks.
- Intentional Crunch deviation: coreutils 6.10 is manually compiled utility-by-utility with TinyCC/musl instead of a stock configure/make install path.
- Required failure semantics: every library object and declared utility must compile and install, and smoke checks must pass, before the output contract is satisfied.
