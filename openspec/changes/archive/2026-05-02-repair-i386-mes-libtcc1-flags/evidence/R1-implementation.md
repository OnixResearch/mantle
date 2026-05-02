# R1 implementation

Task-ID: R1
Covers: bootstrap.i386-mes-libtcc1-flags.repair

Patched `bootstrap/spike-i386-mes-runtime-layout.ncl` so `runtime_libtcc1_object` compiles Mes `lib/libtcc1.c` with `HAVE_FLOAT=0`, `HAVE_FLOAT_STUB=0`, `HAVE_LONG_LONG=0`, and `TCC_TARGET_I386=1`. This removes the prior `constant exceeds 32 bit` / rc 139 failure and avoids the placeholder archive path.
