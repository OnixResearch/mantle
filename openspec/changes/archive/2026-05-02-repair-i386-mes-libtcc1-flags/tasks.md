# Tasks: Repair i386 Mes libtcc1 compile flags

## Repair

- [x] R1 Patch the i386 runtime-layout proof to build Mes `libtcc1.c` with 32-bit-safe flags. [covers=bootstrap.i386-mes-libtcc1-flags.repair]
- [x] R2 Run Crunch build evidence showing `runtime_libtcc1_object` and archive succeed. [covers=bootstrap.i386-mes-libtcc1-flags.evidence]
- [x] R3 Record the next blocker after real `libtcc1.a` creation. [covers=bootstrap.i386-mes-libtcc1-flags.next-blocker]
- [x] R4 Verify and archive the OpenSpec change. [covers=bootstrap.i386-mes-libtcc1-flags.repair]
