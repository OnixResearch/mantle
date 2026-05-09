# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.bison.3.4.1

- Audited `bootstrap/bison-3.4.1-musl.ncl` source pin, declared TinyCC/musl predecessor inputs, manual config, object compilation loops, archive creation, install copy, and output checks.
- Intentional Crunch deviation: Bison is manually compiled with early TinyCC/musl and a generated `config.h` rather than relying on a stock host configure/make path.
- Required failure semantics: object compilation, archive creation, final link, data installation, and smoke checks must fail closed.
