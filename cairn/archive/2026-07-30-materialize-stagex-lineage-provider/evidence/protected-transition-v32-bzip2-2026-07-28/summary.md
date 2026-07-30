# Protected StageX transition through bzip2 1.0.8

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built bzip2 1.0.8.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v32-bzip2-20260728`
- Test task: pueue `2440`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1114.91 seconds
- Report status: `complete`
- Plan BLAKE3: `0bde7b931bf3717213723a853f077da9a1d4b6b89436d92a1264ca82919453fc`
- Manifest BLAKE3: `495b2e97a6095eaed6180c3359acac3333cc08eb62a14c6e70990af92fbc843c`
- Source-bundle manifest BLAKE3: `e9112bde5d990d68945cb91183911f0acd2d04cfd67ea59501177c196cc7fda5`
- Source-state BLAKE3: `7d27636b3c6f43d52d2b9b665a641a3979d49506c337633e9b9216ad58de51a0`
- Planned stages: 51
- Allowed protected execution events: 644
- Denied protected execution events: 0
- Fallback events: 0

## bzip2 boundary

The transition authenticated bzip2 1.0.8 source identity `fixed-url-29051c18498ee931a89eb96a6c890e06b4098c8ec2e0f4242f0d8337192a8070` with content BLAKE3 `969e26e40644aaed6137a7403c29397ada624f6dc20d38dfae5fec9d2079292d`.

The source-built GNU sed executable applied three bounded compatibility rewrites. The protected TinyCC executable compiled nine source files and linked two executables. The smoke boundary checked help output, completed a compression roundtrip, and rejected malformed compressed input.

- Compatibility header BLAKE3: `3bf37f6c622b15c4f6acada147c5ca5921da1cfefe9b0cb6dc53c5d10a10de13`
- Configured source BLAKE3: `a3e8d977c0652889d2714f7eb9d1cfbe48581c05c7b7e3b79405334504fedd73`
- `bzip2-1.0.8`: `b6911b1a367e8b159264078e7976d363e3cb9d268a1dd074ddca51a76916b8a7`
- `bzip2recover-1.0.8`: `f4bf3e19bd09bcd64503def65eb7bd7eaa5ea43af94536b8d6e638a48501e0a0`
- Roundtrip output: `b37fa30b2c16ae67d4e023b19fbecf3b10dab989e5fdedec510c29780cc5793b`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through bzip2 1.0.8 only. It does not prove coreutils, the later shell, musl and GCC toolchain, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next conventional frontier is coreutils 5.0. Its bounded utility set and source authority are not yet part of the protected plan.
