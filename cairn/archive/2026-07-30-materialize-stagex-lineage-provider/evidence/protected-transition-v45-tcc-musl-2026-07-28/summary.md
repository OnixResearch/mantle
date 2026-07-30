# Protected transition v45: musl-linked TinyCC

The fresh create-new transition completed through the first TinyCC 0.9.27 compiler linked by the Mes-runtime predecessor and configured for reduced musl 1.1.24.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v45-tcc-musl-20260728`
- Pueue task: `2907`
- Test result: `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1777 filtered out; finished in 1192.09s`
- Protected stages: 65
- Protected events: 1161 allowed; zero denied
- Fallback events: zero
- Plan BLAKE3: `9a37c4c17397977c5c7986edbe384b8ecfab938d0787b3f63e9caecf4702476b`
- Manifest BLAKE3: `8f8882b4469304f9b185a1001bbd68d4fef33ceddf680e9829bd32eb2a8fff37`
- Source-state BLAKE3: `fd8d6777751e1a0eaf86d17578f8b43c71c8059f03cdb07cb1933e486a96a855`
- Configured source BLAKE3: `679e40759989e2bf0886a430ac3cb00ef22b045dd18fa50fdeddba44230c7674`
- Compiler and alias BLAKE3: `053f36a66503797b3aaee40d2b0962dc643554e4ecf35d00ee2b7f947601f2af`
- Runtime archive BLAKE3: `0e8b75458ad70ab03142b27a3014af68f57c03004f4d22a65702e60d531140e9`
- Positive object BLAKE3: `f7d6cd4379debd6239bbbcfffdd602d87ad9e8cfabe322902a85fa5ace902fc6`
- Positive linked binary BLAKE3: `92ebe6be234ab289a07f5560ce0f25b1a696917d4515d2ec92c330fa0cd3a8a4`

The stage used two predecessor build executions and four new-compiler smoke executions. The negative compile returned a nonzero status and produced no object.

This evidence proves only the bounded musl-linked TinyCC handoff. It does not prove a complete libc, the later GNU/GCC/binutils chain, or normalized provider admission.
