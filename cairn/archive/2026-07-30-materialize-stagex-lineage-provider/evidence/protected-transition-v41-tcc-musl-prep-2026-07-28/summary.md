# Protected StageX transition through TinyCC musl-prep

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through the TinyCC-to-musl preparation compiler.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v41-tcc-musl-prep-20260728`
- Test task: pueue `2838`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1383.37 seconds
- Report status: `complete`
- Plan BLAKE3: `68a8f7e3f0ffa6f157b7eb042b7c7b2d271843ffecf08df5234262a8f3ec65ae`
- Manifest BLAKE3: `1ea9b7334d547fb34fc21e88b6c5e898e0cb153868ee655c2d36366ffcaaafd0`
- Source-bundle manifest BLAKE3: `a5793a83c98d36f673f77ef0083d402db32f4fd96e765bd657b89c088d1bf4a6`
- Source-state BLAKE3: `01c47ecd6d7270f0314b03843b4f7c0c7f4ede86de28f3c01ef55fa988c47782`
- Planned stages: 61
- Allowed protected execution events: 933
- Denied protected execution events: 0
- Fallback events: 0

## TinyCC-to-musl boundary

The bridge reuses authenticated TinyCC 0.9.27 source and the checked StageX compatibility patch. It also binds `bootstrap/tcc-musl-prep.ncl` with BLAKE3 `88f0a242b6e53d3618383af6d9fce83bbacaa4cf4f2403bba56baf9651901b4e`.

Bounded Rust logic added the exact fixed-width type aliases required by the Mes-linked predecessor. TinyCC 0.9.27 compiled and linked the bridge with explicit CRT, libc, `va_list.o`, and `libtcc1.a` paths. The output carries the exact Mes headers and runtime needed until musl exists. Positive and malformed object-compilation smokes ran through the generated bridge executable.

- Configured source BLAKE3: `29489150a78ce433dac24acfaf58ec66a4cd37d3eb93e223c5b9150f5c71a0aa`
- `tcc-musl-prep`: `4add5639d2016d79b1ac258aa502098fc941354ded0acd91bf8cafea42240a77`
- Carried Mes libc: `bd46a4a350e17ec4a7870be6c6dea82f7327c513ee4e2720bebb7b6e26f20d85`
- Carried Mes headers: `2577c43f136d52c8d7b61ba8f7cab614b31898586d3fab634bebe6e1fd14cb7d`
- Positive object: `41c08a115f275f00099aa397872090612e853be0fb5f13ba55ddb89737e42377`

Two earlier create-new attempts hit the old five-minute Mes process limit before this bridge. Their v39/v40 plans and failure audits remain in `protected-transition-v39-mes-timeout-2026-07-28/`. The successful v41 run used the reviewed ten-minute named limit.

## Non-claim and next blocker

This evidence proves the bounded protected lineage through the TinyCC-to-musl preparation compiler only. It does not prove musl 1.1.24, the later GNU/GCC/binutils closure, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical frontier is musl 1.1.24. Its authenticated source, generated compatibility sources, output identities, and closed protected audit are not yet part of the transition.
