# Validation: Add determinism regression harness

## Commands

### Targeted determinism coverage

```sh
nix shell nixpkgs#bubblewrap -c bash -lc 'export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/nix/store/6jafhh81cf85d0vqwrnhl5yfc4wibxvq-protobuf-29.6/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH" && export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig:${PKG_CONFIG_PATH:-}" && export SNIX_BUILD_SANDBOX_SHELL=/bin/sh && export CARGO_TARGET_DIR=/home/brittonr/git/crunch/crunch/target/determinism-check && cargo test -p crunch-pipeline --test integration_build pipeline_determinism_ -- --nocapture && cargo test -p crunch resolve_bwrap_source_ -- --nocapture'
```

Observed result snippets:

- `test result: ok. 4 passed; 0 failed; 4 ignored; 0 measured; 10 filtered out; finished in 0.80s`
- `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 137 filtered out; finished in 0.00s`

### OpenSpec validation

```sh
openspec validate add-determinism-regression-harness
```

Observed result snippet:

- `Change 'add-determinism-regression-harness' is valid`

## Notes

- The self-build-friendly determinism case now runs as a real pipeline build in `crates/crunch-pipeline/tests/integration_build.rs`.
- That probe uses a helper derivation as a build dependency and discovers it through `$NIX_STORE/*-determinism-bootstrap-tool`, matching the self-build-style mounted-input pattern without requiring a full self-build proof run.
