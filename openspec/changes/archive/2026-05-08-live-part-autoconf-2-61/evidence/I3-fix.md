# I3 source/output contract hardening evidence

Task-ID: I3
Covers: bootstrap.part.autoconf.2.61

- Added source-pin comments beside `src`: `provenance: fosslinux/live-bootstrap parts.rst section "autoconf 2.61"` and `first-consumer: bootstrap/autoconf-2.61.ncl`.
- Replaced `./configure ... 2>/dev/null || true` with fail-closed `./configure --prefix="$out"`.
- Replaced `make install ... || fallback-copy` with fail-closed `make -j1 install prefix="$out"`.
- Strengthened output checks to require `bin/autoconf`, `bin/autoreconf`, `bin/autoheader`, `bin/autom4te`, and `share/autoconf`.
