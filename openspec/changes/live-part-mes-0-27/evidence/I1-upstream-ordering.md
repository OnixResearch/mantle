Task-ID: I1
Covers: bootstrap.part.mes.0.27

# I1: Upstream ordering and output contract

Command/context:

```sh
sed -n '202,216p' ~/git/pi-repos/fosslinux--live-bootstrap/parts.rst
sed -n '1,220p' ~/git/pi-repos/fosslinux--live-bootstrap/steps/mes-0.27.1/pass1.kaem
find ~/git/pi-repos/fosslinux--live-bootstrap/steps/mes-0.27.1 -maxdepth 2 -type f -print
```

Source: upstream `parts.rst` section `mes 0.27` and `steps/mes-0.27.1/pass1.kaem`.

Result: upstream builds GNU Mes 0.27.1 after the stage0-posix/M2-Planet tools exist. The part unpacks Mes plus NYACC 1.00.2, patches `wait4.c`, creates arch include shims, removes pregenerated `psyntax` files, builds `mes-m2` through `kaem.${MES_ARCH}`, regenerates NYACC parser tables with `mes-m2`, materializes `mescc.scm`, and builds Mes libc artifacts.

Expected Crunch output contract for `bootstrap/mes.ncl`:

- `bin/mes-m2`
- `bin/mescc.scm`
- `lib/x86_64-mes/x86_64.M1`
- `lib/x86_64-mes/crt1.o`
- `lib/x86_64-mes/libmescc.a`
- `lib/x86_64-mes/libc.a`
- `lib/x86_64-mes/libc+tcc.a`
- `lib/x86_64-mes/libtcc1.a`
- `lib/linux/x86_64-mes/elf64-header.hex2`
- `include/mes-include/`
- `mes/module/`
- `lib/M2libc/`

Status: complete. `bootstrap/mes.ncl` scope matches the upstream Mes 0.27.1 boundary plus Crunch's normalized output layout for downstream TinyCC stages.
