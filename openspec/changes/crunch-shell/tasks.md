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

- [ ] Add `hook | String | optional` field to `mkShell` params in `builders/mk_derivation.ncl`
- [ ] Make `mkShell` builder script write `$out/.crunch-shell.json` with version, env, path_entries, hook
- [ ] Remove the "this derivation is not meant to be built" error — mkShell must actually build to produce the sidecar
- [ ] Sidecar `path_entries` populated from `buildInputs` bin dirs
- [ ] Sidecar `env` populated from the `env` param (not build-sandbox env, not derivation `env` field)
- [ ] Stdlib test: `mkShell { env = { X = "1" }, hook = "echo hi" }` produces valid sidecar JSON
- [ ] Stdlib test: `mkShell` without hook produces sidecar with null/absent hook
- [ ] Stdlib test: sidecar version is 1

## PR 3: CLI and imperative shell

- [ ] Add `Shell` variant to CLI `Command` enum with `--command`, `--run`, `--with`, `--no-hook`, `--strict-hooks`
- [ ] Keep `Develop` as an alias that dispatches to the shell handler
- [ ] Implement `cmd_shell()` in `src/shell_cmd.rs`: build target → read sidecar → snapshot host env → call `compute_activation()` → match `ExecTarget` → `std::process::Command`
- [ ] Wire `--command` / `--run` mutual exclusion as a clap conflict
- [ ] Wire `--with` resolution: store paths validated on disk, `.#attr` resolved and built via existing project build path
- [ ] Wire hook execution: `$SHELL -c <hook>` before exec, warn on non-zero, fatal with `--strict-hooks`
- [ ] Wire `--no-hook` suppression (shell-side, not core-side)
- [ ] Propagate command exit code as process exit code

## PR 4: Integration tests

- [ ] Build a shell with env vars, assert `--command env` output contains them
- [ ] Build a shell with buildInputs, assert `--command which <tool>` finds the tool
- [ ] Build a shell with hook, assert hook output appears before command output
- [ ] Test `--no-hook` suppresses hook output
- [ ] Test `--strict-hooks` with failing hook exits without running command
- [ ] Test `--with <store-path>` adds PATH entry before sidecar entries
- [ ] Test missing sidecar (plain derivation target) produces clear error naming `.crunch-shell.json`
- [ ] Test `--command` exit code propagation (non-zero)
- [ ] Test `crunch develop` alias dispatches identically to `crunch shell`

## Validation

- [ ] Audit `crates/crunch-shell/Cargo.toml` dependencies — no I/O crates
- [ ] Confirm `src/shell_cmd.rs` has no env merging, PATH dedup, or hook decision logic
- [ ] Run `cargo test -p crunch-shell` (pure core tests pass without bwrap/store)
- [ ] Run integration tests with `--command env` to verify end-to-end activation
