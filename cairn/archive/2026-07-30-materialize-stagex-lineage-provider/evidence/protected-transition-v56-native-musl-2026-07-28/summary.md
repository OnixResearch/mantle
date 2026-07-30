# Protected transition v56: native musl

The fresh create-new transition completed through a static, upstream-shaped musl 1.1.24 candidate. The protected self-hosted TinyCC compiled 708 source files and created the static archive. The declared TinyCC musl-v2 predecessor compiled 19 assembly and math files. It also supplied the malformed-source diagnostic.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v56-native-musl-20260728`
- Pueue task: `3409`
- Test result: `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1798 filtered out; finished in 1334.59s`
- Protected stages: 75
- Protected events: 2141 allowed; zero denied
- Native musl events: 732
- Fallback events: zero
- Plan BLAKE3: `39dce699e4f8da9805262f672ef52aea54cb5e7891b712893fa700e3f5f62310`
- Manifest BLAKE3: `aad7bb539be1e0f401a27a3937ab0a8d3076c8ceb7e644bda3a0e4bac4f36687`
- Recipe BLAKE3: `0f86c28c5a4f7290002200871edede784e79566b4f02c9ebb1f22b654c1e6c1c`
- Configured-source BLAKE3: `b65d4f08a8a49ebc0c402c00f68a9348fb416cb8d66b066b4d08022e744018b2`
- Static libc BLAKE3: `da903202e9b5f574fd32ba550e654b918f5ce4dad0c87b5d70171eaba1f0cf07`
- Header-tree BLAKE3: `315e38bd3f318804adf63fadc9689a9b692ee69986e26c6fddd4670190498700`
- Runtime-smoke BLAKE3: `37b452f13fc423d7a1a5cc061264e4a897b305d144b0e214648738d106cb3e49`
- Validation-receipt BLAKE3: `f578c82fb2c9eb48d87b171daedaef41f63c7f82b64d26b14e6f686f9c3f230e`

The Rust orchestration replaces shell loops, text rewrites, source selection, header installation, and identity checks. It normalizes only archive metadata fields. It does not change member names, member order, or object bytes.

The static runtime smoke compiled, linked, and ran successfully. The predecessor rejected malformed C with a nonzero exit status and a diagnostic. The receipt binds these observations.

This evidence does not prove complete musl behavior, a dynamic runtime, compiler correctness, the later GNU/GCC/binutils chain, or StageX provider admission.
