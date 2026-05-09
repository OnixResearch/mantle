# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.gcc.4.0.4

- Added source provenance and first-consumer comments beside the `gcc_src` fixed-output fetch.
- Removed fallback manual install copying that could preserve partial compiler fragments after `install-gcc` failed.
- Removed suppressed `install-target-libgcc` failure.
- Converted the GCC smoke warning into a fail-closed smoke and required `cc`, `cc1`, and `libgcc.a` output contract artifacts.
