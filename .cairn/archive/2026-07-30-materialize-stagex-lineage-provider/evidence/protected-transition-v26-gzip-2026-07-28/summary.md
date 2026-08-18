# Protected StageX transition through gzip 1.2.4

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built gzip 1.2.4.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v26-gzip-20260728`
- Test task: pueue `2218`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1085.60 seconds
- Report status: `complete`
- Plan BLAKE3: `7421dd433a1c5e1876a9dc32a48ca3ed47cd5325630fde9fdfe8185dc7251f7b`
- Manifest BLAKE3: `9a69d780220d6d4bec40b63ddccc80441098779ac9f1586d8544c8101f422359`
- Source-bundle manifest BLAKE3: `6d6e2f0ae906c3a421b90e220fed77e6f90b556cd7982731352ce28f39f5a796`
- Source-state BLAKE3: `048cafd02200a6776f846dffdfd48c08554ac06b077db1ba6ba9370fa957b056`
- Planned stages: 44
- Allowed protected execution events: 575
- Denied protected execution events: 0
- Fallback events: 0

## gzip boundary

The transition authenticated gzip 1.2.4 source identity `fixed-url-28390b124a4e4a0426e370860163351efc23ff1d3d3b6be84a5609868b00f45d` with content BLAKE3 `de8516b8bfc78a91f5d37837ff70d6926f4b03cf68924ae7ef6ed602f7f316fd`.

The bounded Rust shell compiled and ran a checked source-built CRC generator. It then compiled 14 gzip source files with the protected TinyCC 0.9.27 executable and linked gzip. The protected smoke boundary checked help output, completed a gzip/gunzip roundtrip, and rejected malformed compressed input.

- Configured source BLAKE3: `9c052723a21ae2449b3c6f0aa3caf252519f532b3dd26fdcc65d8b08bdf16848`
- `gzip-makecrc`: `dd10648de513f76a1ecdbfa23cc88953f8c6e46848311a6372de7498aed43c6e`
- Generated CRC table: `19a3f07c4b071fe898927d1da2c3b64d37e02b0e8cf309d2a9fc4d70ac9d8fa1`
- `gzip-1.2.4` and `gunzip-1.2.4`: `d2939461715a694222c1a612c0908754e599eb2d268fe012c525845367eff04a`
- Roundtrip output: `78abaccbae9efa7039ff82d80215d2fbd3c4cd0221dadd85dd9df731f15d8751`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through gzip 1.2.4 only. It does not prove the remaining bootstrap utilities, the later musl and GCC toolchain, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical live-bootstrap stage is GNU tar 1.12. The current protected source authority does not yet include that stage. The next exact step is to export and bind its authenticated fixed source, then materialize tar under the same closed protected authority.
