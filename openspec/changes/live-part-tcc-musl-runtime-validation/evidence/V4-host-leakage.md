# V4 host leakage scan

The successful focused validation of `bootstrap/musl-1.1.24-tcc.ncl` reported no leakage findings. The downstream `bootstrap/tcc-musl.ncl` failure also reported no leakage findings before failing inside `tcc-0.9.27-musl.drv`.

Evidence:
- `V2-musl-1.1.24-tcc-passed-validation-summary.json`
- `V2-tcc-musl-after-musl-validation-summary.json`
