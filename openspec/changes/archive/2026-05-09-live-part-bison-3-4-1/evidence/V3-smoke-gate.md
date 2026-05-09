# V3 bison 3.4.1 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.bison.3.4.1

The smoke contract is conditional on a produced `bison-3.4.1-musl` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed Bison smoke success is claimed.

The hardened derivation now fails closed unless `bin/bison`, `share/bison/yacc.c`, and the installed `bison --version` smoke are produced during a real build.
