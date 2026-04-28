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
- The output contract is asserted by the derivation before success with `test -x` checks for `hex0`, `kaem`, `M2-Planet`, `M1`, `hex2`, and `blood-elf`.
- The install step copies the downstream contract into `$out/bin` plus `$out/lib/M2libc`; no downstream bootstrap part needs to know the upstream in-tree build paths.

Status: complete. No code edit was required; the existing derivation already carries the self-contained source pins and output contract needed by this part.
