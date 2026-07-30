# Protected transition v48: second-musl

The fresh create-new transition completed through reduced musl 1.1.24 rebuilt by the protected musl-linked TinyCC compiler.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v48-musl-pass2-20260728`
- Pueue task: `3063`
- Test result: `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1779 filtered out; finished in 1390.56s`
- Protected stages: 67
- Protected events: 1387 allowed; zero denied
- Fallback events: zero
- Plan BLAKE3: `20b6cf5b1b160dbed464476e526270c34c588c5dfbb6ccff7d16cdfacffb7e04`
- Manifest BLAKE3: `1d3427d724a1e6839862a98f4e45feff70cee602edbf13a80a6698c1138d0740`
- Source-state BLAKE3: `fd8d6777751e1a0eaf86d17578f8b43c71c8059f03cdb07cb1933e486a96a855`
- Recipe BLAKE3: `5f85fc84eb8eb235ca15d3179db8a3504668804f140d46f15fe7af2e53a0dec5`
- Configured source BLAKE3: `575e19d9e8f5d23be1ae0e87b9e40bebbaa3be7845632980586f38ef2d906e5d`
- `libc.a` BLAKE3: `86f238f807b2580bcb814ef914a64a288e34bb89101c8b5f094cd2a124febaed`
- `crt1.o` BLAKE3: `c34258edb3d4de07a67e1d20c7dc5946543ce4ba078ace182b0b99cd4bbb7257`
- Header tree BLAKE3: `4d5a63f48ef27c8377a5319d3d68cd1a2b724a974ee5d8281a67f8fb0b5dc1c7`
- Linked smoke BLAKE3: `30c18329ab3981639dba0dbdef48cd01beae52abab89c8b732f8f6eb1248fc57`

The stage compiled 219 libc sources and three CRT sources. It performed one archive operation and three compile/link/negative-smoke operations. The final audit matched every event against the supervisor’s canonical exact path-and-digest authorization.

This evidence proves only the bounded second-musl handoff. It does not prove complete libc behavior, later GNU/GCC/binutils stages, or normalized provider admission.
