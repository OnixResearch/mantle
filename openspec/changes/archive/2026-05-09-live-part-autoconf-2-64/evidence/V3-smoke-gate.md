# V3 autoconf 2.64 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.autoconf.2.64

The smoke contract for this part is conditional on a produced `autoconf-2.64` output. Because V2 has no output path while `make-3.82-tcc` remains blocked, no `autoconf`, `autoreconf`, `autoheader`, or `autom4te` smoke success is claimed.

The hardened derivation now fails closed unless those entrypoints and `share/autoconf` are installed.
