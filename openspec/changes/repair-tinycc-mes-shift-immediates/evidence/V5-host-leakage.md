Task-ID: V5
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# Host-leakage scan

Result: PASS. The scan found only declared Mes runtime output paths under `$out/lib/mes`, `$PREFIX/lib/mes`, or copied predecessor Mes runtime inputs. It found no undeclared host compiler, host libc, host `/usr/bin`, host `/bin/{cc,gcc,ld,ar}`, `ld-linux`, `libc.so`, Nix command, or Nix gcc/binutils/glibc store path.

Command:

```sh
PAT='/usr/bin|/bin/(cc|gcc|ld|ar)|/lib(64)?/|ld-linux|libc\.so|nix-|nix |/nix/store/.+-(gcc|binutils|glibc)'
grep -nE "$PAT" \
  bootstrap/tinycc-mes.ncl \
  bootstrap/tinycc.ncl \
  openspec/changes/repair-tinycc-mes-shift-immediates/evidence/I2-fix.md \
  openspec/changes/repair-tinycc-mes-shift-immediates/evidence/V3-tinycc27-build-full.log \
  openspec/changes/repair-tinycc-mes-shift-immediates/evidence/V4-smoke-full.log || true
```

Reviewed hits:

```text
bootstrap/tinycc-mes.ncl: $PREFIX/lib/mes, $MES_OUT/lib/${MES_ARCH}-mes, $out/lib/mes, CONFIG_TCCDIR/CRTPREFIX/LIBPATHS under $out/lib/mes
bootstrap/tinycc.ncl: $PREFIX/lib/mes, $TCC26/lib/mes, $out/lib/mes, compile_tcc27 runtime paths under $out/lib/mes and $out/include/mes
```

Conclusion: all matches are declared bootstrap/runtime artifacts inside the Crunch derivation graph. No undeclared host path or host tool match was present.
