# Protected StageX transition through GNU tar 1.12

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built GNU tar 1.12.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v27-tar-20260728`
- Test task: pueue `2233`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1087.99 seconds
- Report status: `complete`
- Plan BLAKE3: `f8bf80caf3b60d8c680775711e4b51445131630c50cbeb432028a722760a6e1b`
- Manifest BLAKE3: `5ace1556501a43692de8b10df10ecdb600e1039048583dfe15d8fb01a624b2c5`
- Source-bundle manifest BLAKE3: `fb693cea29637174c37ab949f1436831a1c0096eb3610231ef296549d291ae55`
- Source-state BLAKE3: `5b544dbc1798a3dd092773ee4a0780f601be9b4d53ab5bfddb60a4450c80ac90`
- Planned stages: 46
- Allowed protected execution events: 609
- Denied protected execution events: 0
- Fallback events: 0

## GNU tar boundary

The transition authenticated GNU tar 1.12 source identity `fixed-url-56372665c26229fd9923e24885bba78d12de4459e7fb19f3ee6d5cb21dd1f1ac` with content BLAKE3 `0cbc269e6eccf79f32b9f64204a0277a0e6df877dc1a6d0fd0d3e240365abdff`.

The bounded Rust shell generated checked configuration and a source-bound date-parser stub. It compiled 29 source files with the protected TinyCC 0.9.27 executable and linked GNU tar. The protected smoke boundary checked version output, completed a create/extract roundtrip, and rejected malformed archive input.

- Configured source BLAKE3: `e3bbe80418be6a87402866f8215926c9e3a7054d745a0f46be852306dff790c7`
- `tar-1.12`: `57c861b961bfc2c1047ab7f175c0f16024ea91787ba5a5e43600c721b5eff6ca`
- Roundtrip output: `59392bf4bef63f5688ff8604868a22adcecfe94f75c1234fe7c9df1107038be2`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through GNU tar 1.12 only. It does not prove the remaining bootstrap utilities, the later musl and GCC toolchain, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical live-bootstrap stage is GNU sed 4.0.9. The current protected source authority does not yet include that stage. The next exact step is to export and bind its authenticated fixed source, then materialize sed under the same closed protected authority.
