# Design: 32-bit-safe Mes libtcc1 flags

Mes `lib/libtcc1.c` contains conditional long-long and float helper definitions. The `tcc26-i386` predecessor is a narrow i386 bootstrap compiler; compiling those broad helpers caused `constant exceeds 32 bit` and rc 139. For the runtime-layout proof, compile `libtcc1.c` with:

- `HAVE_FLOAT=0`
- `HAVE_FLOAT_STUB=0`
- `HAVE_LONG_LONG=0`
- `TCC_TARGET_I386=1`

This yields a real `libtcc1.o` and archive for the next handoff probe, separating the fixed runtime-library construction from the still-failing TinyCC 0.9.27 object emission.
