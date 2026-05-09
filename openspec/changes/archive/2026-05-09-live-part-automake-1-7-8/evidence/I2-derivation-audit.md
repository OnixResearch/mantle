# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.automake.1.7.8

- Audited `bootstrap/automake-1.7.8.ncl` source pin and declared inputs.
- Intentional Crunch deviation: derivation uses Crunch fixed-output `fetchTarball` and store-input discovery via `find_input` instead of upstream shell-global variables.
- Required failure semantics: configure/install failures must propagate; fallback-copy install behavior is not acceptable runtime proof.
