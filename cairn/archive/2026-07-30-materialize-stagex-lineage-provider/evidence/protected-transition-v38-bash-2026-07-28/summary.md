# Protected StageX transition through bash 2.05b

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built bash 2.05b.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v38-bash-20260728`
- Test task: pueue `2721`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1149.96 seconds
- Report status: `complete`
- Plan BLAKE3: `ac9a158b8e629dff0be6f0e25805070daa57bd65315adb1f06f044c0c0b55edc`
- Manifest BLAKE3: `f5a0b8f0bc14ec40a54f6a6329114d8d0f0438a126f8c7c8771dde215fc1b7e3`
- Source-bundle manifest BLAKE3: `a5793a83c98d36f673f77ef0083d402db32f4fd96e765bd657b89c088d1bf4a6`
- Source-state BLAKE3: `01c47ecd6d7270f0314b03843b4f7c0c7f4ede86de28f3c01ef55fa988c47782`
- Planned stages: 59
- Allowed protected execution events: 928
- Denied protected execution events: 0
- Fallback events: 0

## Bash boundary

The transition authenticated bash 2.05b source identity `fixed-url-62e1b90422b8fcc2101590edd36dc56e7a72e8bc233854bb1e8b63dc0f76a6c0` with content BLAKE3 `130da251856869c8d04a52d35299181bc6c3be8e38624e731a9323382c3d755e`.

Bounded Rust logic extracted ten checked helper sources from `bootstrap/bash-2.05b-tcc.ncl`. Their individual BLAKE3 identities are part of the lineage manifest. TinyCC 0.9.27 compiled 92 source files and linked bash with the exact Mes CRT, libc, `libgetopt.a`, and `libtcc1.a`. The positive smoke wrote `bash-ok`. The negative smoke rejected malformed shell syntax.

- Configured source BLAKE3: `acd33bededf76dc149f7c7fa14a2e1a1246c5dafe2ef65eee18cf0d82e3d6bcd`
- `bash-2.05b`: `f7326a0fcc0878e7cd3616640cee376f71bab5583460b5459cac91168809c03f`
- `sh-2.05b`: `f7326a0fcc0878e7cd3616640cee376f71bab5583460b5459cac91168809c03f`
- Smoke output: `f9bb7985b1ac20ccbfdd1195397be795edb7996ab02e6f173d166517c19eb579`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through bash 2.05b only. It does not prove the musl handoff, GCC/binutils closure, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical frontier is the TinyCC-to-musl handoff. The protected manifest does not yet contain the `tcc-musl-prep` compiler, musl 1.1.24 source and runtime identities, or their closed audit authorization.
