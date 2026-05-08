# V3 autoconf 2.52 output smoke gate

Task-ID: V3
Covers: bootstrap.part.autoconf.2.52

The output-contract smoke is conditional on V2 producing an `autoconf-2.52` output path. V2 has no output path because the part is prerequisite-gated on the `make-3.82-tcc` runtime/build blocker.

Future promotion must prove, on a real output, that:

- `bin/autoconf` exists and runs under the declared bootstrap runtime;
- `bin/autoreconf` exists;
- `bin/autoheader` exists;
- `bin/autom4te` exists;
- `share/autoconf` exists.

No Autoconf command smoke success is claimed here.
