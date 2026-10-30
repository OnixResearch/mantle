# Source acquisition evidence

Only dependency/source acquisition and lock setup were executed in this slice.

`git ls-remote --tags https://github.com/bytecodealliance/wrpc.git 'refs/tags/v0.*'`
resolved v0.15.0 to peeled commit `aa01409ed38bf7a978570ff567c4475376925df6`.

`nix store prefetch-file --json https://codeload.github.com/bytecodealliance/wrpc/tar.gz/aa01409ed38bf7a978570ff567c4475376925df6`
returned `sha256-C1MFw8/ninRC5oqZc/Ff0Ymu2dZNdrrLSo3DfAUP+JI=`.
The corresponding `--unpack` acquisition returned
`sha256-atDMHeLJ72NRyglNDhaqB7cD+KyBbI0QDbRGGgEkxjg=`.

Inspection of that acquired source's unmodified Cargo.lock found Wasmtime
33.0.0, wrpc-runtime-wasmtime0.29.0, wrpc-transport0.28.4,
wit-bindgen0.36.0, and wit-bindgen-wrpc0.10.0. No git dependency sources
were present in that lock. The actual CLI source confirms
`wit-bindgen-wrpc rust --world WORLD --out-dir OUTPUT WIT`.

`cargo generate-lockfile --manifest-path packages/wrpc-reference/guest/Cargo.toml`
resolved 43 packages, retaining wit-bindgen0.36.0. Its generated lock is kept
verbatim; the source-line cap does not justify hand-editing mechanical metadata.

There is no build/test/format/lint execution evidence, generated guest digest,
materialization receipt, remote RPC observation, publication identity, lifecycle
gate result, or completion claim for those parent-owned execution steps.
