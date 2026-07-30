# Protected StageX transition through reduced musl 1.1.24

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through the first reduced musl 1.1.24 static runtime.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v43-musl-20260728`
- Test task: pueue `2866`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1383.13 seconds
- Report status: `complete`
- Plan BLAKE3: `83abdd99d15a89c30710b3949f9427b7d50e6b974ff894bbb3325ce4c72f26cc`
- Manifest BLAKE3: `9cbda4203a9803575ac9c183ea64720b9a3cdf320f4445ccfd52897642e52fc8`
- Source-bundle manifest BLAKE3: `d3ed9b10c763444d416182b58b2e4b766ed80f0a8ab71d07c0bcf576e599052d`
- Source-state BLAKE3: `fd8d6777751e1a0eaf86d17578f8b43c71c8059f03cdb07cb1933e486a96a855`
- Planned stages: 63
- Allowed protected execution events: 1155
- Denied protected execution events: 0
- Fallback events: 0

## First-musl boundary

The transition authenticated musl 1.1.24 source identity `fixed-url-e5130672fe4e4cdd9b19e8602ae7cbe70696d8de4b08732ff6bc01fa3c9b7d85` with content BLAKE3 `2abcaf25adf8f529b9f45e5549bfaa5cc5827305624c081e9388ac2ad828ba17`.

Bounded Rust logic removed the declared source subsets, generated `alltypes.h`, `syscall.h`, and the version header, and materialized 24 independently bound compatibility sources from the checked recipe. The TinyCC musl-prep compiler compiled 215 libc objects and three CRT objects. It created the static libc archive and linked a smoke binary with exact CRT, libc, and `libtcc1.a` paths. Malformed C was rejected.

- Configured source BLAKE3: `d1d3f1b8c7b99ba3902a4b999f55ee4ba3410cfd7ebac9462176f2d1d7565dcf`
- `libc.a`: `b3a7f8bd311e8d3eaa6b1767e87276d3d02dce04a95808ed187082b05bb6211d`
- `crt1.o`: `701c5afce9f5e09d0bcca0702764c64148fcacfe7e1477e2321b249b03af66be`
- Installed headers: `4d5a63f48ef27c8377a5319d3d68cd1a2b724a974ee5d8281a67f8fb0b5dc1c7`
- Linked smoke binary: `92ebe6be234ab289a07f5560ce0f25b1a696917d4515d2ec92c330fa0cd3a8a4`

## Non-claim and next blocker

This evidence proves the bounded reduced first-musl static handoff only. It does not prove a complete libc, dynamic runtime, the later GNU/GCC/binutils closure, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The next canonical frontier is TinyCC 0.9.27 rebuilt against musl. Its musl-linked compiler identity, runtime behavior, and closed protected audit are not yet part of the transition.
