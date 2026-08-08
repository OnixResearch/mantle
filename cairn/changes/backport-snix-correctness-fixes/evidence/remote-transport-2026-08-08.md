# Remote transport fixes

## Implemented boundaries

The Snix HTTP service now treats a binary-cache URL as a directory base. Endpoint joins preserve configured subpaths.

The Snix HTTP path and Mantle pull path now decode all zstd frames.

## Coverage

Tests cover matching signed narinfo, mismatched requested identity, malformed narinfo, and zero NAR requests on metadata rejection.

URL tests cover subpath bases with and without a trailing slash. Existing Mantle tests cover query and fragment removal.

Zstd tests cover two valid frames, a truncated later frame, malformed input, decoded bytes beyond the NAR limit, and zero local `PathInfo` after rejection.

## Validation

```text
cargo test -p snix-store --lib nix_http
22 passed; 0 failed

cargo test -p crunch-store --lib zstd_
5 passed; 0 failed

cargo test -p snix-store --lib
93 passed; 0 failed

cargo test -p crunch-store --lib
345 passed; 0 failed

cargo clippy -p crunch-store --lib --no-deps -- -D warnings
Finished successfully.

cargo fmt --check -p snix-store -p crunch-store -v
Finished successfully.

git diff --check
Finished successfully.
```
