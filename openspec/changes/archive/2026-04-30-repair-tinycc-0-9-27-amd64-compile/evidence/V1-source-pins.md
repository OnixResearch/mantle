Task-ID: V1
Covers: bootstrap.part.tinycc.0.9.27.amd64.compile

# Source-pin audit

Result: PASS.

Command:

```sh
CARGO_TARGET_DIR=target/cargo-script-source-pins \
  cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl
```

Output:

```text
source-pin audit: 1 files, 1 fetch blocks, 0 issues
```

Full transcript: `evidence/V1-source-pins-full.log`.
