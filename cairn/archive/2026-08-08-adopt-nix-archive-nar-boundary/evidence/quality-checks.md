# Quality and contract checks

Each section contains the exact combined standard output and error from the named command.

## Focused Clippy

```text
$ CARGO_TARGET_DIR=/tmp/mantle-nar-clippy-target SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo clippy -p crunch-nar -p crunch-store -p mantle --tests --no-deps -- -D warnings
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
```

## Boundary guard self-test and scan

```text
$ nix develop -c cargo -Zscript scripts/check-nar-boundary.rs --self-test
nar-boundary self-test: PASS
$ nix develop -c cargo -Zscript scripts/check-nar-boundary.rs
nar-boundary guard: PASS (5694 files)
```

## Retained fixture identity test

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-nar --test parity retained_fixture_manifest_binds_source_and_payload_identities
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/crates/crunch-nar)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.41s
     Running tests/parity.rs (/home/brittonr/.cargo-target/debug/deps/parity-96602c267ffe7560)

running 1 test
test retained_fixture_manifest_binds_source_and_payload_identities ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

```

## Store CLI boundary test

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test integration store_verify_cli
warning: /home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/mantle-adopt-nix-archive-nar-boundary)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.56s
     Running tests/integration.rs (/home/brittonr/.cargo-target/debug/deps/integration-5f53d31bdadfd6c5)

running 1 test
test store_verify_cli_uses_filesystem_nar_observation_and_reports_tampering ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.04s

```

## Empty-home locked offline metadata

```text
$ CARGO_HOME=<empty> CARGO_NET_OFFLINE=true GIT_CONFIG_GLOBAL=/dev/null nix develop -c cargo metadata --offline --locked --format-version 1 --config .cargo/vendor-config.toml
"package":"70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1"
offline locked metadata: PASS
```

## Nickel machine contracts

```text
$ nickel export --format json <fixture-and-evidence-contracts>
Nickel machine contracts: PASS
```

## Whitespace check

```text
$ git diff --check
git diff --check: PASS
```

Quality-check verdict: PASS
