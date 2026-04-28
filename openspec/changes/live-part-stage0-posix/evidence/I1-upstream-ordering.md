Task-ID: I1
Covers: bootstrap.part.stage0.posix

# I1: Upstream ordering and output contract

Command/context:

```sh
sed -n '12,158p' ~/git/pi-repos/fosslinux--live-bootstrap/parts.rst
find ~/git/pi-repos/fosslinux--live-bootstrap/seed -maxdepth 1 -type f -print
```

Source: `~/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, sections `bootstrap-seeds` through `mescc-tools-extra`, plus `seed/{seed.kaem,after.kaem,preseeded.kaem}`.

Result: upstream starts from `bootstrap-seeds`, uses the stage0-posix chain for `hex0`, `kaem-optional`, `hex1`, `hex2`, `M0`, `M1`, `catm`, `blood-elf`, and full `kaem`, then builds `M2libc`, `M2-Planet`, `M2-Mesoplanet`, and `mescc-tools-extra`.

Expected Crunch output contract for `bootstrap/stage0-posix.ncl`:

- `bin/hex0`
- `bin/hex1`
- `bin/hex2`
- `bin/M0`
- `bin/M1`
- `bin/catm`
- `bin/blood-elf`
- `bin/kaem`
- `bin/get_machine`
- `bin/M2-Planet`
- mescc-tools-extra helpers: `cp`, `chmod`, `mkdir`, `untar`, `ungz`, `unbz2`, `unxz`, `sha256sum`, `match`, `replace`, `wrap`
- `lib/M2libc/`

Status: complete. `bootstrap/stage0-posix.ncl` scope matches the upstream early seed/tool boundary named by this part change.
