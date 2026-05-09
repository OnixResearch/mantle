# V2 Heirloom devtools build gate evidence

Task-ID: V2
Covers: bootstrap.part.heirloom.devtools

No successful `crunch build bootstrap/heirloom-devtools.ncl` is claimed in this closeout.

The TinyCC/Mes predecessor path remains prerequisite-gated by the archived `libtcc.c rc=139` boundary. Therefore Heirloom devtools cannot yet be promoted as trusted source-built parser-generator providers.

No host yacc/lex, Nix-provided yacc/lex, or legacy tool output was substituted as evidence.
