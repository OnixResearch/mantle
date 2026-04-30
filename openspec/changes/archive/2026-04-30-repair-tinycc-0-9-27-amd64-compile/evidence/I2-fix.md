Task-ID: I2
Covers: bootstrap.part.tinycc.0.9.27.amd64.compile

# TinyCC 0.9.27 amd64 compile repair

Result: PASS.

Implemented in `bootstrap/tinycc.ncl`.

Root cause found during I2:

- `tinycc 0.9.26` miscompiled power-of-two byte shifts/divisions in the TinyCC 0.9.27 x86_64 byte emitter.
- Generated code for `o(unsigned int c)` contained `shr $0`, so `o(0xb8)` never reduced `c`, emitted bytes forever, repeatedly grew `.text`, and made `tcc -c hello.c` hang.
- The same hazard affected `gen_le16`, `gen_le32`, `gen_le64`, REX byte composition, and `ret n` high-byte emission.

Repair:

- Patch `x86_64-gen.c` during bootstrap to emit bytes through `unsigned char *` indexing instead of `>> 8` / power-of-two division.
- Patch `REX_BASE` and REX-byte expressions to avoid constant shifts/multiplies.
- Keep existing tcc-0.9.26 allocator-growth normalizations for `* 2` forms.
- Make the malformed-input path fail cleanly by replacing the final stderr `fprintf` with `fputs` and returning before unsafe cleanup after an error.

Positive and negative proof is in:

- `evidence/V2-build-full.log`
- `evidence/V3-smoke-full.log`
