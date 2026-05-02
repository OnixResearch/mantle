# Design: i386 Mes runtime/header layout spike

The derivation imports the existing `spike-i386-tinycc26-cross-smoke.ncl` output and Mes/TinyCC sources. It creates a scratch runtime tree with:

- Mes headers copied under `include/mes/include`;
- `include/arch/*` populated from Mes `include/linux/x86`;
- Mes config header for non-system-libc mode;
- i386 `crt1.o` compiled by `tcc26-i386`;
- attempted `libtcc1.a` generation by `tcc26-i386`.

If `libtcc1.c` compilation fails, the derivation records the failure and creates a placeholder archive from the proven `crt1.o` only to continue a header-oriented TinyCC 0.9.27 object probe. This keeps the two facts separate: partial layout creation can work, while a complete runtime library remains blocked.
