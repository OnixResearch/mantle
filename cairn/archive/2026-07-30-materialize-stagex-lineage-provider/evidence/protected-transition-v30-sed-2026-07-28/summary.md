# Protected StageX transition through GNU sed 4.0.9

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built GNU sed 4.0.9.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v30-sed-fresh-20260728`
- Test task: pueue `2331`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1199.54 seconds
- Report status: `complete`
- Plan BLAKE3: `9c7185530c8ce09d9d0c78a03a91ffae792e3800885c33a80e54ae8a262fc1bd`
- Manifest BLAKE3: `be89b70de54fa4f7c4b67761992e8eb590a16fcfd4d04f066d1c149b61a5707d`
- Source-bundle manifest BLAKE3: `58180e33d97d4dd12f15a0b8998f19ee66cff60056d1535e2d2a62d8256005b6`
- Source-state BLAKE3: `84185a03664bc45746f029cfd82ad97af181e40d19e5d68d194d3806bbbc7fe0`
- Planned stages: 48
- Allowed protected execution events: 626
- Denied protected execution events: 0
- Fallback events: 0

## GNU sed boundary

The transition authenticated GNU sed 4.0.9 source identity `fixed-url-2861bfe2bbe8003326313432362cdd09a73a549a874abb706bce6ba4e5378011` with content BLAKE3 `65aa1a57a2966a247b4992de6525e319adb99f8ad2fe4cc2eb543a28740994ad`.

The bounded Rust shell compiled 13 source files and linked GNU sed with the protected TinyCC 0.9.27 executable. The protected smoke boundary checked version output, completed an in-place substitution, and rejected a malformed expression.

- Configured source BLAKE3: `3e96d4952cf6c52707082fc120d9a372def665c73bef7ec13609abbfc201b99c`
- `sed-4.0.9`: `cc4bc898fbad65d9c0a07eab85c61c99abdb9d7d7d747948067bea7d9954618a`
- Substitution output: `91f02abc6d98e0f00da4f3b1249ae0acd29544f7b19304f190d29f523885c077`

## Failed fresh attempts

The retained v28 scratch reached GNU sed but exposed an audit-origin expectation error. The implementation now checks the original TinyCC 0.9.27 authorization identity.

The retained v29 scratch failed before GNU sed when one `mes-m2` process exceeded its fixed 300000 ms limit. It is not completion evidence. The successful v30 run used a separate create-new root.

## Non-claim and next blocker

This evidence proves the bounded protected lineage through GNU sed 4.0.9 only. It does not prove bzip2, coreutils, the later musl and GCC toolchain, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next conventional frontier is bzip2 1.0.8. Its source and source-rewrite steps are not yet bound into the current protected source authority and plan.
