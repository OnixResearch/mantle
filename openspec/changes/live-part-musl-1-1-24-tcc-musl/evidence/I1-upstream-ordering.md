Task-ID: I1
Covers: bootstrap.part.musl.1.1.24.tcc.musl

# musl 1.1.24 (tcc-musl) upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `musl 1.1.24 (tcc-musl)`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/musl-1.1.24/pass2.sh` (symlink to `pass1.sh`) and `pass3.sh` for follow-up context.

Ordering:

1. First musl (`musl-1.1.24-tcc`) and musl-linked tcc (`tcc-musl`) exist.
2. musl 1.1.24 is rebuilt with the just-built musl-linked tcc.
3. This rebuild fixes early libc issues before the `tcc-musl-v2` compiler rebuild.

Expected Crunch output contract:

- `lib/libc.a`
- installed musl headers such as `include/stdio.h`
- static startup/runtime objects expected by downstream tcc/musl stages

Negative space:

- This is the second musl pass, not the later v3/v4 regeneration path.
- Early generated-header/complex-source restrictions from upstream must still be accounted for or justified.
