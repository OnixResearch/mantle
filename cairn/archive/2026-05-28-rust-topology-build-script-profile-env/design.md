## Context

`build_script_child_env()` currently provides structural Cargo env values but does not provide profile/runtime values used by cc-rs. cc-rs treats missing `OPT_LEVEL` as a hard error during compiler selection. Cargo also provides `DEBUG` and `NUM_JOBS`, which common build scripts and helper crates use for debug flags and bounded parallelism.

## Decisions

### 1. Derive profile env in pure helpers

**Choice:** Add pure helpers that map Mantle profile strings into `OPT_LEVEL`, `DEBUG`, and `NUM_JOBS` values.

**Rationale:** Keeps Cargo profile parity testable without executing build scripts or reading ambient env.

### 2. Keep parallelism deterministic

**Choice:** Set `NUM_JOBS=1` for now.

**Rationale:** Mantle does not yet expose a native Rust topology job count. A fixed single-job value is deterministic and avoids ambient Cargo state.

### 3. Scope to current cc-rs blocker

**Choice:** Do not add broad Cargo env surfaces such as rustflags, encoded makeflags, or tool-specific CFLAGS in this change.

**Rationale:** The self-probe blocker is `OPT_LEVEL`; additional envs should be added only when evidence demands them.

## Risks / Trade-offs

- Profile mapping is currently bounded to Cargo defaults: dev/test-like profiles map to `OPT_LEVEL=0`, `DEBUG=true`; release/bench map to `OPT_LEVEL=3`, `DEBUG=false`.
- Custom Cargo profile settings are not modeled yet.
