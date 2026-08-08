# Package checks

Date: 2026-08-08

## `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo check -p snix-store -p snix-castore -p snix-tracing -p crunch-store`

```text
warning: /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-tracing)
    Checking nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/nix-compat)
    Checking crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-attestation-core)
    Checking crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-action-result-core)
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-overlay-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-gc-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-repair-core)
    Checking snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-castore)
    Checking crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-attestation)
    Checking crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-nar)
    Checking snix-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-build)
    Checking snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-store)
    Checking crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-store)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.40s
```

Exit status: `0`

## `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt --check -p snix-store -p snix-castore -p snix-tracing -p crunch-store -v`

```text
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-store/src/lib.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-store/tests/authority_source_policy.rs"
[custom-build (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-castore/build.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-castore/src/lib.rs"
[custom-build (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-store/build.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-store/src/lib.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-tracing/src/lib.rs"
rustfmt --edition 2024 --check /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-store/src/lib.rs /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/crates/crunch-store/tests/authority_source_policy.rs /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-castore/build.rs /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-castore/src/lib.rs /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-store/build.rs /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-store/src/lib.rs /home/brittonr/git/OnixResearch/mantle/.pi/worktrees/backport-snix-i4-20260808/vendor/snix-tracing/src/lib.rs
```

Exit status: `0`
