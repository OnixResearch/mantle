# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.diffutils.2.7

- Added source provenance and first-consumer comments beside the `diffutils_src` fixed-output fetch.
- Replaced bare installed-entrypoint tests with explicit fail-closed error messages.
- Added installed `diff`/`cmp` smokes for equal files and intentional different-file detection.
