Task-ID: I1
Covers: bootstrap.part.tcc.musl.prep

# tcc-musl-prep upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, sections `tcc 0.9.27 (patched)`, `musl 1.1.24 and musl_target`, and `tcc 0.9.27 (musl)`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/tcc-0.9.27/pass2.sh` and `pass3.sh`.

Ordering:

1. `stage0-posix`, `mes 0.27`, `tinycc 0.9.27`, `make`, and `sed` exist.
2. The chain prepares a patched/configured tcc 0.9.27 before the first musl build.
3. The musl-facing tcc is configured with musl include/lib/interpreter paths while still bridging from prior compiler/runtime outputs.
4. The next stages use that compiler to build `musl 1.1.24` and then rebuild tcc against musl.

Expected Crunch output contract:

- `bin/tcc`
- `bin/tcc-musl-prep`
- carried `lib/x86_64-mes/` for linking until musl exists
- carried `include/mes/` headers for the bridge period

Negative space:

- This is a bridge compiler, not the final self-hosted `tcc-musl-v2`.
- Upstream `/usr/lib` and `/usr/include` locations must map to `$out` and explicit downstream inputs in Crunch.
