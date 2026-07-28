# Protected StageX transition through oyacc 6.6

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built oyacc 6.6.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v36-oyacc-20260728`
- Test task: pueue `2687`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1153.53 seconds
- Report status: `complete`
- Plan BLAKE3: `37a5c43f35e5ccca69dea79733da359e5d76b05d7e43dd51453ad8660ef0a5cc`
- Manifest BLAKE3: `c8cc61fd6d6380903dc9ae850ad3295cd281991a05baf43f8caa3bf93e2b9bdf`
- Source-bundle manifest BLAKE3: `8a58a015b0a1f45023ed38f92dc46f5a733be983794524117abe68654786ac5f`
- Source-state BLAKE3: `fea81936ea62a0fde1950e657bb7d9b76a5fd3d8fbbe2df19f0e9cfa5e54f474`
- Planned stages: 57
- Allowed protected execution events: 832
- Denied protected execution events: 0
- Fallback events: 0

## oyacc boundary

The transition authenticated oyacc 6.6 source identity `fixed-url-a674f552ef5345d19cb7e37f3c6f80f38dcf1a2f54b188894680bda6c6930c1a` with content BLAKE3 `b1ea74a498a000f5f27160860fd44042d738295c78c1db56a4e0fdd10ea5ee76`.

GNU patch 2.5.9 applied the checked live-bootstrap Mes-libc and TinyCC compatibility patches. TinyCC 0.9.27 compiled 13 source files and linked the parser generator with the exact Mes runtime and `libgetopt.a`. The positive smoke generated `y.tab.c`. The negative smoke rejected a checked malformed grammar.

- Configured source BLAKE3: `f4cde58315dcbfc49c47928f9258debd030140c54e249cb2a11beef1eba30296`
- `oyacc-6.6`: `0b073649ea023167a293495f25302cc3debc4dd6727d9214ce6be9361503aa50`
- Generated parser output: `6e719a5fc57aa5e8a06b7f7f37c6fdf672cd2ef2f91c27a7ca1daff128ac65a2`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through oyacc 6.6 only. It does not prove bash 2.05b, the musl and GCC toolchain closure, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical frontier is bash 2.05b. Its authenticated source, protected build, output identity, and audit closure are not yet part of this transition.
