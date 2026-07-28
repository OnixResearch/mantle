# Protected StageX transition through coreutils 5.0

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through a bounded source-built coreutils 5.0 utility set.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v35-coreutils-20260728`
- Test task: pueue `2574`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1204.69 seconds
- Report status: `complete`
- Plan BLAKE3: `88c07025b669894c69c0fdac0a2814eab4671af5c3cf29028deb508291891668`
- Manifest BLAKE3: `791bfb656c82a61550dce3418010e0d8674c28058cd4f271ff8bcf7412bed183`
- Source-bundle manifest BLAKE3: `11a116c4d1a406e46c3e2f15f11f8e643abcd387fc6668e1680abe1d230bfe42`
- Source-state BLAKE3: `69abdb0caabd2b45024548ca48c821f9205e2701515908fed236875ead99ce30`
- Planned stages: 54
- Allowed protected execution events: 814
- Denied protected execution events: 0
- Fallback events: 0

## coreutils boundary

The transition authenticated coreutils 5.0 source identity `fixed-url-758798ffe110eb26e73f9f7832f4addd66c8b8722387fba4513bb786c8528d76` with content BLAKE3 `82d3dd8a0cbb5b5032512d747b5bb79b5dfdfd94b2cd11d9ae50a1dd0b05d814`.

GNU patch 2.5.9 applied eight checked compatibility patches. TinyCC 0.9.27 compiled 96 library sources and 34 utility sources, created one support archive, and linked 25 utilities. The bracket alias is an exact executable copy of `test`. Six smoke executions checked directory creation, output, copying, reading, file testing, and rejection of a missing copy source.

- Configuration header BLAKE3: `03f96124348df7f962ee7c2e23ecfb97a7eeadcbd32a50e503850d5d0c6513b0`
- Configured source BLAKE3: `232ccf0c23c2e8a028c5ca42cbb46f3afa1aad664a5d5187e11fecdcb04334d4`
- Bounded binary outputs: 26
- Smoke output BLAKE3: `fd65316f83e13de60a964675b6ab49ee8bd0e40db3aef391c8c1cbee240375d8`

Exact per-binary paths, sizes, and BLAKE3 values are in `coreutils-inventory.json` and `transition-report.json`.

## Non-claim and next blocker

This evidence proves the bounded protected lineage through the selected coreutils 5.0 utilities only. It does not prove the remaining conventional utilities, a later shell, musl and GCC toolchain closure, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical frontier is oyacc 6.6. Its authenticated source, protected build, output identity, and audit closure are not yet part of this transition.
