# V2 bison 3.4.1 build gate evidence

Task-ID: V2
Covers: bootstrap.part.bison.3.4.1

## Result

No successful `crunch build bootstrap/bison-3.4.1-musl.ncl` is claimed in this closeout.

## Blocker

`bootstrap/bison-3.4.1-musl.ncl` depends on the TinyCC/Mes/musl and Make 3.82 predecessor chain, plus Bison 2.3/Flex/M4 bootstrap tools. Multiple predecessors are archived only with prerequisite-gated evidence, and Make 3.82 remains a known runtime promotion blocker. This change therefore closes source-hardening/output-contract work and preserves build proof as prerequisite-gated.

## Trust boundary

No host Bison, Nix-provided Bison, host GCC, or legacy compiler output was substituted as evidence for this part.
