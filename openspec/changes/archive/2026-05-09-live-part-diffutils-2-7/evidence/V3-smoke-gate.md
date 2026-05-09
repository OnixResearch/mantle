# V3 diffutils 2.7 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.diffutils.2.7

The smoke contract is conditional on a produced `diffutils-2.7-musl` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed diffutils smoke success is claimed.

The hardened derivation now fails closed unless `bin/diff`, `bin/cmp`, equal-file checks, and different-file detection checks pass during a real build.
