# Tasks: crunch shell

## PR 1: Pure environment core crate

- [x] Create `crates/crunch-shell/` with `Cargo.toml` depending only on `serde`, `serde_json`, `thiserror`
- [x] Define `ShellSidecar` (version, env, path_entries, hook) with serde deserialization and version validation
- [x] Define `HostEnv` as a plain snapshot struct (env map, shell path)
- [x] Define `ActivationPlan` (env `BTreeMap`, PATH `Vec<PathBuf>`, hook `Option<String>`, `ExecTarget`, warnings)
- [x] Define `ExecTarget` enum: `Interactive`, `Command`, `Run`
- [x] Define `ShellWarning::ProtectedVarSkipped { key }` and `ShellError` variants
- [x] Implement `compute_activation(sidecar, host_env, with_paths) -> Result<ActivationPlan, ShellError>`
- [x] Assert protected vars (HOME, USER, TERM, LOGNAME, DISPLAY, LANG, SHELL) are never overwritten from sidecar
- [x] Assert PATH is non-empty after composition
- [x] Assert PATH dedup uses first-occurrence-wins
- [x] Assert `CRUNCH_SHELL` is always set in the plan env map
- [x] Unit tests: env merge, protected var skip with warning, PATH ordering, PATH dedup, hook passthrough, missing version, bad version, empty sidecar defaults, round-trip serialization, ExecTarget variants

## PR 2: Sidecar generation in mkShell

- [x] Add `hook | String | optional` field to `mkShell` params in `builders/mk_derivation.ncl`
- [x] Make `mkShell` builder script write `$out/.crunch-shell.json` with version, env, path_entries, hook
- [x] Remove the "this derivation is not meant to be built" error — mkShell must actually build to produce the sidecar
- [x] Sidecar `path_entries` populated from `buildInputs` bin dirs
- [x] Sidecar `env` populated from the `env` param (not build-sandbox env, not derivation `env` field)
- [x] Stdlib test: `mkShell { env = { X = "1" }, hook = "echo hi" }` produces valid sidecar JSON
- [x] Stdlib test: `mkShell` without hook produces sidecar with null/absent hook
- [x] Stdlib test: sidecar version is 1

## PR 3: CLI and imperative shell

- [x] Add `Shell` variant to CLI `Command` enum with `--command`, `--run`, `--with`, `--no-hook`, `--strict-hooks`
- [x] Keep `Develop` as an alias that dispatches to the shell handler
- [x] Implement `cmd_shell()` in `src/shell_cmd.rs`: build target → read sidecar → snapshot host env → call `compute_activation()` → match `ExecTarget` → `std::process::Command`
- [x] Wire `--command` / `--run` mutual exclusion as a clap conflict
- [x] Wire `--with` resolution: store paths validated on disk, `.#attr` resolved and built via existing project build path
- [x] Wire hook execution: `$SHELL -c <hook>` before exec, warn on non-zero, fatal with `--strict-hooks`
- [x] Wire `--no-hook` suppression (shell-side, not core-side)
- [x] Propagate command exit code as process exit code

## PR 4: Integration tests

- [x] Build a shell with env vars, assert `--command env` output contains them (requires bwrap)
- [x] Build a shell with hook, assert hook output appears before command output (requires bwrap)
- [x] Test `--no-hook` suppresses hook output (requires bwrap)
- [x] Test `--strict-hooks` with failing hook exits without running command (requires bwrap)
- [x] Test `--with <store-path>` adds PATH entry and tool is found (requires bwrap)
- [x] Test missing sidecar (plain derivation target) produces clear error naming `.crunch-shell.json` (requires bwrap)
- [x] Test `--run` exit code propagation (non-zero) (requires bwrap)
- [x] Test `--command` / `--run` mutual exclusion (CLI-level, no bwrap)
- [x] Test `crunch shell --help` shows all new flags
- [x] Test `crunch develop` alias exists and shows deprecation notice
- [x] Test `--with` with nonexistent path fails early
- [x] Improved `can_build()` to detect missing sandbox shell (skips gracefully)
- [x] Fixed `exec_plan` Command mode to resolve programs against activation PATH

## Validation

- [x] Audit `crates/crunch-shell/Cargo.toml` dependencies — only serde, serde_json, thiserror
- [x] Confirm `src/shell_cmd.rs` has no env merging, PATH dedup, or hook decision logic
- [x] Run `cargo test -p crunch-shell` (21 pure core tests pass without bwrap/store)
- [x] Run full integration tests: 49/51 pass (2 pre-existing build-report failures)
