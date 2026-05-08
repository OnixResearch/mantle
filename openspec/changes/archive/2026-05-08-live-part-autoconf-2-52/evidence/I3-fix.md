# I3 source/output contract hardening evidence

Task-ID: I3
Covers: bootstrap.part.autoconf.2.52

- Added source-pin comments: `provenance: fosslinux/live-bootstrap steps/autoconf-2.52/sources` and `first-consumer: bootstrap/autoconf-2.52.ncl`.
- Replaced suppressed configure fallback with fail-closed `./configure --prefix="$out" CC=tcc CFLAGS="-static"`.
- Replaced suppressed build/install fallback with fail-closed `make -j1 prefix="$out"` and `make -j1 install prefix="$out"`.
- Strengthened output checks to require `bin/autoconf`, `bin/autoreconf`, `bin/autoheader`, `bin/autom4te`, and `share/autoconf`.
