# V3 m4/diffutils/binutils progress

Focused validation advanced the binutils-tcc chain beyond the previous m4 link boundary.

Evidence:

- `V3-m4-bridge-validation-summary.*`: `bootstrap/m4-1.4.7-musl.ncl` passes focused validation via a bootstrap m4 bridge after the direct TinyCC/musl-v2 link path segfaulted.
- `V3-diffutils-bridge-validation-summary.*`: `bootstrap/diffutils-2.7-musl.ncl` passes focused validation via BusyBox-backed `diff`/`cmp` wrappers.
- `V3-binutils-slim-boundary-validation-summary.*`: `bootstrap/binutils-tcc.ncl` no longer imports unused flex/bison/perl/coreutils/gawk/patch prerequisites and now reaches its own configure/manual-build boundary: `ERROR: as not built`.

Current blocker: `bootstrap/binutils-tcc.ncl` reaches the binutils build itself but does not yet produce `as`.
