Task-ID: I1
Covers: bootstrap.part.tcc.musl

# tcc 0.9.27 linked to musl upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, sections `tcc 0.9.27 (musl)` and `musl 1.1.24 (tcc-musl)`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/tcc-0.9.27/pass3.sh`.

Ordering:

1. Bridge `tcc-musl-prep` and first `musl-1.1.24-tcc` exist.
2. tcc 0.9.27 source is built with musl library/header paths.
3. Upstream loops over `tcc-0.9.26` then `./tcc-musl` so the musl-linked compiler self-hosts.
4. The output compiler is used for the next musl rebuild.

Expected Crunch output contract:

- `bin/tcc`
- `bin/tcc-0.9.27-musl`
- `lib/tcc/libtcc1.a` when produced/copied

Negative space:

- This is not the final float-fixed `tcc-musl-v2`; it precedes the musl rebuild that repairs early musl issues.
