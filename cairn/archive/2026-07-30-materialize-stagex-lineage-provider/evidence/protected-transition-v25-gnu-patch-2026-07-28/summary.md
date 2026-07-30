# Protected StageX transition through GNU patch 2.5.9

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built GNU patch 2.5.9.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v25-gnu-patch-20260728`
- Test task: pueue `2238`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1077.08 seconds
- Report status: `complete`
- Plan BLAKE3: `8c92a14c174b8ab7810c40535aa614ec7727a6614cc744419d1516129d290841`
- Manifest BLAKE3: `54f218fe753d7cf23542fdb54ffb308ff00eae2abae63a0649e68da182a7f078`
- Source-bundle manifest BLAKE3: `7ec4786f81326dc6633479ff78ed53cfd18eb5086ae7374113fdef4921f78751`
- Source-state BLAKE3: `3b9d2b015f8b15f73c39c89c5f04ad2cbea983bb4dabf0921945983f7bf46a12`
- Planned stages: 40
- Allowed protected execution events: 554
- Denied protected execution events: 0
- Fallback events: 0

## GNU patch boundary

The transition authenticated GNU patch 2.5.9 source identity `fixed-url-08fc9bbfa8cbf77e5bbfca6cffcfe7116d4927854bfc2b9352b912a00e017545` with content BLAKE3 `d7dd692b74dea83bc6b6aed6efbd9f874aa991a1ba5722e8f4b06bf0c20a21e0`.

The bounded Rust shell generated the checked configuration, compiled 19 source files with the protected TinyCC 0.9.27 executable, and linked GNU patch. The protected smoke boundary checked version output, applied a real patch, and rejected malformed patch input.

- Configured source BLAKE3: `ae44c6060b51d532b264a5fb2bcf0fa6541493482cbf1a5b402f1605ea1596f3`
- `gnu-patch-2.5.9`: `2193ea653bfbc16acc2609a5c7ef6c0b03a7a65dc50e6525d23868598e25fbcb`
- Positive smoke output: `79d1d8da0b625035cdbfc9d51841030861b9f4cf7c5abbe442a8d13efc352170`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through GNU patch 2.5.9 only. It does not prove the remaining bootstrap utilities, a general shell, the later musl and GCC toolchain, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical live-bootstrap stage is gzip 1.2.4. The current protected source authority does not yet include that stage. The next exact step is to export and bind its authenticated fixed source, then materialize gzip under the same closed protected authority.
