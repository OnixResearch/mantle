# V2 gates evidence

## Commands

```bash
openspec validate release-verification-key-show
openspec_gate stage=design change=release-verification-key-show
openspec_gate stage=tasks change=release-verification-key-show
cargo test -p crunch --test release_cli attest_ -- --nocapture
```

## Results

- `openspec validate release-verification-key-show`
  - `Change 'release-verification-key-show' is valid`
- `openspec_gate stage=design change=release-verification-key-show`
  - `PASS`
- `openspec_gate stage=tasks change=release-verification-key-show`
  - `PASS`
- `cargo test -p crunch --test release_cli attest_ -- --nocapture`
  - `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.06s`
  - Re-ran the broader attestation CLI slice after adding `key-show`.

## Summary

The change remains OpenSpec-valid, both design/tasks gates pass, and the full
attestation CLI regression slice stays green with the new trusted-key export
command in place.
