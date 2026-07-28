# Bootstrap Seeds

Auditable binary seeds for the crunch from-source bootstrap chain.

## AMD64/hex0-seed

The 229-byte hex0 assembler for AMD64 from
[stage0-posix](https://github.com/oriansj/stage0-posix).
This is the sole binary trust root -- every other tool in the chain is
built from source starting from this seed.

hex0 reads a text file of hex byte pairs (with comments) on stdin and
writes the corresponding binary on stdout. It is the simplest possible
assembler: no labels, no macros, no relocations. A programmer can
audit the 229 bytes by hand against the annotated source in
`hex0_AMD64.hex0`.

### Provenance

- **Source repo:** <https://github.com/oriansj/bootstrap-seeds>
  path `POSIX/AMD64/hex0-seed`
- **License:** GPL-3.0-or-later
- **SHA-256:** `66c95985e668f20f2465c2b876f83fef066fd7c8c2dd3adb51a969f2d7120c8b`
- **BLAKE3:**  `cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80`
- **Size:** 229 bytes

### Audit expectations

The hex0 source (`hex0_AMD64.hex0`) is a line-by-line annotated hex dump.
Assembling it with any hex0 implementation produces the seed binary.
Verify that the source comments match the ELF header layout and instruction
semantics. Then confirm that the binary matches the source bytes.
