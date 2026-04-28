Task-ID: I3
Covers: bootstrap.part.stage0.posix

# I3: Self-contained source pins and output contract

Command/context:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/stage0-posix.ncl
sed -n '1,260p' bootstrap/stage0-posix.ncl
```

Result:

- Source-pin audit passed for `bootstrap/stage0-posix.ncl`: `1 files, 7 fetch blocks, 0 issues`.
- Inputs are all declared Nickel inputs; no source path is discovered from the host outside the declared `fetchGit` blocks.
- V3 smoke initially caught an output-contract gap: `hex1` and `M0` were built under `AMD64/artifact/` but the install loop only copied tools from `AMD64/bin/`.
- Fixed `bootstrap/stage0-posix.ncl` to copy early tools from `AMD64/artifact/` as a fallback and to assert `hex1` and `M0` before success.
- The output contract is now asserted by the derivation before success with `test -x` checks for `hex0`, `hex1`, `hex2`, `M0`, `M1`, `kaem`, `M2-Planet`, and `blood-elf`.
- The install step copies the downstream contract into `$out/bin` plus `$out/lib/M2libc`; no downstream bootstrap part needs to know the upstream in-tree build paths.

Status: complete after the install-loop fix.
