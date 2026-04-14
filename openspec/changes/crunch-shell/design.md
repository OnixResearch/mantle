# Design: crunch shell

## Context

The current `crunch develop` flow is:

```
resolve project target → build shell derivation → prepend $out/bin to PATH → exec $SHELL
```

`exec_shell()` in `src/main.rs` hard-wires one env var (`CRUNCH_DEV_SHELL`),
reads `$SHELL` from the host, and ignores everything `mkShell` declared in
`env`, `buildInputs`, or any future hook. Testing this requires spawning a
real subprocess.

`mkShell` in `builders/mk_derivation.ncl` already accepts `env` and
`buildInputs`, computes a PATH, and embeds them in the derivation's `env`
field. But the derivation `env` is for the *build* sandbox, not the user's
interactive session. Conflating the two means shell metadata is invisible to
the Rust exec path.

## Goals / Non-Goals

**Goals:**

- make env vars, PATH, and hooks from `mkShell` reach the user session
- keep environment computation pure and unit-testable without I/O
- support `--command` / `--run` for scripted use
- support `--with` for ad-hoc input composition
- keep the exec boundary as thin as possible: read → call core → exec
- avoid coupling the sidecar format to the derivation contract
- avoid coupling the core to any specific shell or exec mechanism

**Non-Goals:**

- redesign `mkShell` builder phases or change derivation hashing
- add container-based or sandboxed shells
- add direnv export (future change that depends on this one)
- implement shell-specific integrations (bash completions, fish init, etc.)

## Decisions

### 1. Shell metadata travels as a versioned sidecar file

**Choice:** `mkShell` writes `$out/.crunch-shell.json` containing the
activation metadata. The Rust shell command reads this file after the build.

**Rationale:** The derivation contract (`lib/derivation.ncl`) is a closed
record consumed by the glue layer. Embedding shell-specific metadata in the
derivation's `env` field couples the derivation hash to presentation concerns
and makes the metadata invisible to tooling that does not understand the
encoding convention. A separate sidecar keeps the derivation contract stable,
the metadata machine-readable, and the format independently evolvable.

**Schema:**

```json
{
  "version": 1,
  "env": { "RUST_LOG": "debug", "PGHOST": "localhost" },
  "path_entries": [
    "/crunch/store/...-ripgrep/bin",
    "/crunch/store/...-fd/bin"
  ],
  "hook": "echo 'welcome to myproject'"
}
```

- `version` is a required positive integer. The core rejects unknown versions
  with a clear error.
- `env`, `path_entries`, and `hook` all default to empty/null when absent.
- Unknown fields are ignored (forward compatibility).

**Trade-off:** One extra file read after build. Negligible cost.

### 2. Environment computation is a pure function in its own crate

**Choice:** A new `crates/crunch-shell/` crate with no I/O dependencies.
The public API is a single function:

```rust
pub fn compute_activation(
    sidecar: &ShellSidecar,
    host_env: &HostEnv,
    with_paths: &[PathBuf],
) -> Result<ActivationPlan, ShellError>
```

`HostEnv` is a plain struct snapshot of the host environment, not a live
`std::env` read. `ActivationPlan` describes the full environment, PATH,
hook, and exec argv — it performs nothing.

**Rationale:** Tiger Style FCIS. The function takes values in and returns
values out. No `std::env::var`, no `std::fs`, no `std::process`, no async.
Unit tests exercise the full matrix of merging, conflicts, and composition
with plain structs. The imperative shell (in `src/shell_cmd.rs`) reads the
sidecar, snapshots the host env, calls the core, and execs the result. No
logic in the shell beyond plumbing.

**Crate boundary enforcement:** `crates/crunch-shell/Cargo.toml` depends
only on `serde`, `serde_json`, and `thiserror`. No `std::fs`, no `tokio`,
no `crunch-build`, no `crunch-store`. The dependency list is the proof that
no I/O leaks in.

### 3. Protected variables are a static list in the core

**Choice:** `HOME`, `USER`, `TERM`, `LOGNAME`, `DISPLAY`, `LANG`, and
`SHELL` are protected. The core never overwrites them from sidecar data.
If the sidecar declares a protected var, the core records it as a
`ShellWarning::ProtectedVarSkipped { key }` in the activation plan, not a
hard error.

**Rationale:** Hard errors on protected vars would break shells that
naively pass through the build env. Warnings give visibility without
blocking the session. The protected list is a `const` array, not a
runtime config — changing it requires a code change and a test update.

### 4. PATH composition uses explicit segment ordering

**Choice:** Final PATH = `[--with /bin dirs] ++ [sidecar path_entries] ++
[host PATH entries]`. Each segment is an ordered list. Duplicates are
removed (first occurrence wins). The core asserts the result is non-empty.

**Rationale:** Explicit ordering prevents surprises. First-wins dedup
means `--with` always shadows sidecar entries, and sidecar entries always
shadow host tools. The ordering is documented and tested.

### 5. `--with` takes store paths or project attribute references

**Choice:** `--with /crunch/store/...-foo` adds a path directly. `--with
.#bar` resolves the project attribute `bar`, builds it if needed, and adds
its output. Multiple `--with` flags accumulate in declaration order.

**Rationale:** Covers the two common ad-hoc composition cases without
inventing a resolution language. Resolution and building happen in the
imperative shell before calling the core — the core only sees resolved
`PathBuf` values.

### 6. Hooks are data in the core, execution in the shell

**Choice:** The core returns the hook string in `ActivationPlan.hook:
Option<String>`. The imperative shell decides how to run it (`$SHELL -c
<hook>` by default). `--no-hook` suppresses execution entirely. A non-zero
hook exit warns by default; `--strict-hooks` makes it fatal.

**Rationale:** Hooks are side effects. The core describes them as data. The
shell interprets them. This keeps the core pure and makes hook behavior
testable without subprocess exec.

### 7. `--command` and `--run` are exec mode selectors

**Choice:** `--command <argv...>` execs the command directly.
`--run <script>` passes the string to `$SHELL -c`. Both run hooks first
(unless `--no-hook`). Without either flag, an interactive `$SHELL` session
starts. `--command` and `--run` are mutually exclusive.

The core represents the exec target as an enum:

```rust
pub enum ExecTarget {
    Interactive { shell: PathBuf },
    Command { argv: Vec<OsString> },
    Run { shell: PathBuf, script: String },
}
```

The imperative shell matches on this enum and calls `std::process::Command`.

**Rationale:** The core decides *what* to exec based on CLI flags and host
env. The shell does the exec. No decision logic in the shell.

### 8. `crunch shell` replaces `crunch develop`; `develop` becomes an alias

**Choice:** The CLI command is `crunch shell`. `crunch develop` dispatches
to the same handler with no behavioral difference.

**Rationale:** `shell` is shorter, more direct, and matches the Nickel-side
name `devShells`. Keeping `develop` as an alias avoids breaking existing
scripts.

## Risks / Trade-offs

**Sidecar invalidation.** If someone edits `.crunch-shell.json` inside the
store output, activation diverges from the declared shell. Acceptable: store
paths are not writable in normal use, and the sidecar is rebuilt on every
`crunch shell`.

**Hook safety.** Hooks run arbitrary shell code. Same trust model as Nix's
`shellHook` — the project author controls the code. `--no-hook` is the
escape hatch.

**`--with` rebuild cost.** `--with .#big-package` triggers a full build.
Intentional — the user asked for it. No implicit rebuilds happen.

**Protected var list drift.** The static list may need updating for new
terminal protocols or display servers. The cost of adding a var is one line
of code plus one test assertion. Preferable to a config mechanism nobody
configures.
