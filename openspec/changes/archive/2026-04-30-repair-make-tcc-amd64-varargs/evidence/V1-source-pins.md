Task-ID: V1
Covers: bootstrap.part.make.3.82.amd64.execution

Status: pass

## Command

`nix shell nixpkgs#clang -c env CARGO_TARGET_DIR=target cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/make-tcc.ncl bootstrap/tinycc.ncl bootstrap/tcc-musl-prep.ncl bootstrap/tcc-musl.ncl bootstrap/tcc-musl-v2.ncl`

## Result

Exit status: 0
Transcript: `evidence/V1-source-pins-full.log`

The source-pin checker completed successfully. The only emitted text was Cargo's default-edition warning for the cargo-script wrapper.

## Mirror fixes from V2 preflight

- `https://ftpmirror.gnu.org/make/make-3.82.tar.bz2` returned HTTP 502 / connection resets. Switched `bootstrap/make-tcc.ncl` to `https://mirrors.kernel.org/gnu/make/make-3.82.tar.bz2`, which returned HTTP 200 for the same fixed-output tarball/hash.
- `https://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2` returned HTTP 502 during dependency fetch. Switched tcc 0.9.27 consumers to `https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2`, which returned HTTP 200 for the same fixed-output tarball/hash.

Verified: 2026-04-30T23:18:00Z
