# Direct fix summary

## Commands

```sh
cargo update -p rustls-webpki --precise 0.103.12
cargo update -p rand@0.8.5 --precise 0.8.6
cargo update -p rand@0.9.2 --precise 0.9.3
cargo update -p rand@0.10.0 --precise 0.10.1
cargo update -p fastrand
cargo update -p lru --precise 0.16.3
cargo update -p astral-tokio-tar --precise 0.6.0
```

## Result

Directly controlled or tractable findings were fixed before waivers:

- `astral-tokio-tar` -> `0.6.0`
- `lru` -> `0.16.4`
- `rand` -> `0.8.6`, `0.9.3`, `0.10.1`
- `rustls-webpki` -> `0.103.12`
- `fastrand` -> `2.4.1`

## Validation

- `evidence/targeted-cargo-check-after-direct-fixes.txt`
- `evidence/cargo-deny-after-direct-fixes-before-waivers.txt`
- `evidence/cargo-deny-final.txt`
