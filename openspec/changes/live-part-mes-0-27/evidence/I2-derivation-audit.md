Task-ID: I2
Covers: bootstrap.part.mes.0.27

# I2: `bootstrap/mes.ncl` upstream audit

Command/context:

```sh
sed -n '1,320p' bootstrap/mes.ncl
sed -n '202,216p' ~/git/pi-repos/fosslinux--live-bootstrap/parts.rst
sed -n '1,220p' ~/git/pi-repos/fosslinux--live-bootstrap/steps/mes-0.27.1/pass1.kaem
```

Audit result:

- Source scope matches the upstream Mes part: Mes 0.27.1 plus NYACC 1.00.2-lb1 are the only fetched sources.
- Each fetched source is pinned at first consumer with `url`, `hash`, and `name` in `bootstrap/mes.ncl`.
- Crunch intentionally uses `fetchTarball` instead of upstream `ungz`/`untar` distfile steps; the source-pin audit covers the fixed-output boundary, and build logic consumes only declared store inputs.
- Crunch mirrors upstream required transforms: `kaem.run` base-address rewrite, `wait4.c` patch, arch include shims, pregenerated `psyntax` removal, symlink materialization, `kaem.${MES_ARCH}` `mes-m2` build, NYACC parser regeneration, and `mescc.scm` generation.
- Crunch intentionally merges both `mes/module/` and top-level `module/` while preserving Mes-safe shims, because Mes 0.27.1 needs both runtime modules and mescc compiler modules but NYACC/Guile compatibility overlaps can break under `mes-m2`.
- Crunch intentionally normalizes `mescc.scm::assert-system*` for this bootstrap path and builds Mes libc archives with `compiler=mescc`, matching the Mes-specific source variants rather than gcc-flavored inline-assembly sources.
- Crunch output layout is normalized for downstream derivations: `bin/`, `lib/x86_64-mes/`, `lib/linux/x86_64-mes/`, `include/mes-include/`, `mes/module/`, and `lib/M2libc/`.

Status: complete. No code change was required by this audit before running the source-pin and build verification tasks.
