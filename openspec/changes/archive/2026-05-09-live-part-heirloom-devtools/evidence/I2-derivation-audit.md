# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.heirloom.devtools

Audited `bootstrap/heirloom-devtools.ncl`:

- source pin: `http://downloads.sourceforge.net/project/heirloom/heirloom-devtools/070527/heirloom-devtools-070527.tar.bz2`
- expected output contract: installed `bin/yacc` and `bin/lex`
- declared predecessors: TinyCC/musl v2, Make, sed, and stage0
- intentional Crunch deviation: direct fail-closed manual compile fallback is retained as an explicit fallback instead of suppressing missing objects/link failures

No host yacc/lex, Nix-provided yacc/lex, or legacy tool output may be substituted for bootstrap proof.
