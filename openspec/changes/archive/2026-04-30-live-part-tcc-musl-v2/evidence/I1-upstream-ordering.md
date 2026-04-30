Task-ID: I1
Covers: bootstrap.part.tcc.musl.v2

# tcc musl v2 upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `tcc 0.9.27 (musl v2)`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/tcc-0.9.27/pass3.sh` plus surrounding musl rebuild context.

Ordering:

1. First musl-linked `tcc-musl` exists.
2. Second-pass musl (`musl-1.1.24-tcc-musl`) exists.
3. tcc is rebuilt against that more self-consistent musl.
4. Resulting compiler becomes the stable pre-GCC tcc for later tools.

Expected Crunch output contract:

- `bin/tcc`
- `bin/tcc-0.9.27-musl-v2`
- `lib/tcc/libtcc1.a` when rebuilt/copied for downstream links

Negative space:

- This is the definitive tcc-musl handoff for later non-GCC packages, but not a final GCC-class compiler.
