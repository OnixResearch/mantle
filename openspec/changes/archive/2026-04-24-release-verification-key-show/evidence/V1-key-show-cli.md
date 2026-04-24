# V1 key-show CLI evidence

## Command

```bash
cargo test -p crunch --test release_cli attest_key_show_ -- --nocapture
```

## Result

- `cargo test -p crunch --test release_cli attest_key_show_ -- --nocapture`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.01s`
  - Covered:
    - `attest_key_show_explicit_signing_key_prints_trusted_public_key`
    - `attest_key_show_uses_default_config_signing_key`
    - `attest_key_show_missing_signing_key_fails_without_generation`

## Summary

`crunch attest key-show` now succeeds for both explicit and default signing-key
sources, prints verifier-ready trusted public key material, and fails cleanly
without generating a key when no signing key exists.
