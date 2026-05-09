# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.binutils.2.30

- Audited `bootstrap/binutils-tcc.ncl` source pin, wrapper tools, declared predecessors, configure/make invocation, output checks, and smoke checks.
- Intentional Crunch deviation: derivation carries early-bootstrap wrappers and source edits for the TinyCC/Mes/musl handoff instead of a stock host configure environment.
- Required failure semantics: required tool outputs and smoke artifacts must be produced by declared bootstrap inputs; host/Nix binutils substitution is not acceptable proof.
