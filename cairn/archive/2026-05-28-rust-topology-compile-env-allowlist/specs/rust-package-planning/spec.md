## ADDED Requirements

### Requirement: Native topology forwards bounded compile-time env

r[rust_package_planning.native_compile_env_allowlist] Native Rust topology execution MUST forward only explicitly allowlisted parent environment variables needed by direct rustc compile-time `env!(...)` uses.

#### Scenario: allowlisted sandbox shell reaches rustc

GIVEN the parent Mantle process has `SNIX_BUILD_SANDBOX_SHELL` set
WHEN Mantle constructs a native rustc child environment
THEN the child environment MUST include `SNIX_BUILD_SANDBOX_SHELL` with the parent value
AND explicit derivation env values MUST remain authoritative on key collision.

#### Scenario: unrelated ambient variables are rejected

GIVEN the parent Mantle process has an unrelated environment variable such as `LD_PRELOAD` or `SECRET_TOKEN`
WHEN Mantle constructs a native rustc child environment
THEN the child environment MUST NOT include that unrelated variable unless it appears in explicit derivation env.
