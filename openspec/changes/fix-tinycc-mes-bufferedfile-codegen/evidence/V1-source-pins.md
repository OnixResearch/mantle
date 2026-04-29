Task-ID: V1
Covers: bootstrap.part.tinycc.0.9.26.selfcompile

# Source pin audit

Command:

```sh
PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:$PATH" \
  CARGO_TARGET_DIR=$PWD/target/cargo-script \
  cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc-mes.ncl
```

Result: PASS.

Transcript: `evidence/V1-source-pins-full.log`

Summary:

```text
source-pin audit: 1 files, 2 fetch blocks, 0 issues
exit=0
```
