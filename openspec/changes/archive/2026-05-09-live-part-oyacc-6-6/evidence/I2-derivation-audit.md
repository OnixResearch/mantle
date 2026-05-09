# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.oyacc.6.6

Audited `bootstrap/oyacc-tcc.ncl`:

- source pin: `https://github.com/ibara/yacc/releases/download/oyacc-6.6/oyacc-6.6.tar.gz`
- expected output contract: executable `$out/bin/yacc`
- declared predecessors: stage0, Mes, TinyCC, make, and oyacc source
- strengthened smoke boundary: generate `y.tab.c` from a minimal grammar before and after install

No host yacc, Nix-provided yacc, or legacy parser-generator output may be substituted for bootstrap proof.
