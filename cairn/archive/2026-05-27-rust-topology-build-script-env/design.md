## Context

`run_build_script_metadata` currently clears the child process environment, restores only bounded PATH, then sets `OUT_DIR`, `CARGO_PKG_NAME`, and `CARGO_MANIFEST_DIR`. Real build scripts such as `anyhow` require `$RUSTC` and expect relative source probes to run from the package root.

## Decisions

### 1. Build-script env is a pure plan plus shell apply

**Choice:** Add a pure helper that derives deterministic build-script environment key/value pairs from the unit, execution options, and package root. The shell function applies those values to `Command`.

**Rationale:** Keeps the env policy testable without spawning processes.

### 2. Minimal bounded Cargo env

**Choice:** Set `RUSTC`, `HOST`, `TARGET`, and `PROFILE` in addition to the existing `OUT_DIR`, `CARGO_MANIFEST_DIR`, and `CARGO_PKG_NAME`.

**Rationale:** These values are deterministic from the active Mantle invocation and unblock common build-script probes without importing Cargo as the orchestrator.

### 3. Package-root working directory

**Choice:** Run build scripts with current directory set to `CARGO_MANIFEST_DIR` when the source path has a parent.

**Rationale:** Cargo build scripts commonly reference package-relative files such as `src/nightly.rs`.

## Risks / Trade-offs

- More build scripts may require additional Cargo env vars later; this change intentionally limits scope to deterministic values required by the current frontier.
- Changing build-script cwd can alter relative path behavior, but it matches Cargo's documented build-script execution model.
