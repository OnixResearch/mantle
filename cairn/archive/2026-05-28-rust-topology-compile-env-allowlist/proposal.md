# rust-topology-compile-env-allowlist

## Why

After crate disambiguators, clean native topology advances past `object_store@0.13.2` and blocks in vendored `snix-build`: rustc reports `env!("SNIX_BUILD_SANDBOX_SHELL")` is not defined. Mantle clears child rustc environment for hermeticity, but some workspace crates intentionally require a bounded compile-time environment input.

## Change

Add a deterministic allowlist for native rustc compile-time environment variables. The first required key is `SNIX_BUILD_SANDBOX_SHELL`. Mantle should pass only allowlisted variables from the parent process into the rustc child env, while preserving existing explicit derivation env behavior.

## Success

- Pure tests prove allowlisted compile env is forwarded.
- Pure tests prove unrelated ambient variables are rejected.
- Clean self-probe advances beyond the missing `SNIX_BUILD_SANDBOX_SHELL` blocker or records the next deterministic frontier.
