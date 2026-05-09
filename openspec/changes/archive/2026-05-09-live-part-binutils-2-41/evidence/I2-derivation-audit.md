# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.binutils.2.41

- Audited `bootstrap/binutils-full.ncl` source pin, declared full-toolchain predecessors, configure/make invocation, output checks, and smoke checks.
- Intentional Crunch deviation: final full binutils is cross-shaped through Crunch's bootstrapped GCC/musl/binutils predecessors instead of host tools.
- Required failure semantics: missing required tools must fail, not warn and continue.
