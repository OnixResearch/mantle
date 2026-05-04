# tcc-musl after first musl validation

- `bootstrap/musl-1.1.24-tcc.ncl` now passes focused `bootstrap validate` with no leakage findings.
- Downstream `bootstrap/tcc-musl.ncl` now reaches `tcc-0.9.27-musl.drv` and fails there with exit status 139 while compiling/linking TinyCC against the first musl output.
- This advances the blocker from first-musl construction to the musl-linked TinyCC handoff.
- Latest root derivation log: `V2-tcc-musl-after-musl-root-derivation.log`.
