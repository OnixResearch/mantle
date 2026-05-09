# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.coreutils.5.0.musl

- Audited `bootstrap/coreutils-5.0-musl.ncl` source pin, declared TinyCC/musl predecessor inputs, manual config, library object compilation, utility compilation, install layout, and smoke checks.
- Intentional Crunch deviation: early coreutils are manually compiled utility-by-utility with TinyCC/musl instead of a stock configure/make install path.
- Required failure semantics: every declared library object and utility must compile and install, and smoke checks must pass, before the output contract is satisfied.
