Task-ID: V1
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# Source-pin audit

Result: PASS. Both TinyCC bootstrap derivations keep declared source fetch blocks consistent with the source-pin audit.

Command:

```sh
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:$PATH"
CARGO_TARGET_DIR="$PWD/target/check-bootstrap-source-pins-script" \
  cargo -Zscript scripts/check-bootstrap-source-pins.rs \
  bootstrap/tinycc-mes.ncl bootstrap/tinycc.ncl
```

Output:

```text
source-pin audit: 2 files, 3 fetch blocks, 0 issues
```
