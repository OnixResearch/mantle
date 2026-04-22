Evidence-ID: second-wave-functional-core-validation-v2-shell-boundary
Task-ID: V2
Artifact-Type: verification-note
Covers: architecture.nostd.core.crate.boundary.effectful.dependency.outside, functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads, functional.core.shell.adapters.effect.translation.shell.activation.path.translation.in.shell
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

## Commands

```text
cargo test -p crunch-shell-core --lib
cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin -- --nocapture
cargo test -p crunch-shell non_utf8_with_path_is_rejected -- --nocapture
```

## Results

- `cargo test -p crunch-shell-core --lib` → `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- `cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin -- --nocapture` → `test adapter::tests::adapter_preserves_path_order_and_appends_bin ... ok`
- `cargo test -p crunch-shell non_utf8_with_path_is_rejected -- --nocapture` → `test adapter::tests::non_utf8_with_path_is_rejected ... ok`

## Boundary inspection

Files inspected:

- `crates/crunch-shell-core/src/lib.rs`
- `crates/crunch-shell-core/src/plan.rs`
- `crates/crunch-shell-core/src/types.rs`
- `crates/crunch-shell/src/lib.rs`
- `crates/crunch-shell/src/adapter.rs`
- `crates/crunch-shell/src/types.rs`
- `src/shell_cmd.rs`

Observed boundary:

- `crunch-shell-core` now exposes owned-data core APIs only:
  - `parse_shell_sidecar_json(json: String)`
  - `compute_activation(sidecar: ShellSidecar, host_env: HostEnv, output_path: String, with_paths: Vec<String>)`
- `crunch-shell-core` public types stay on owned UTF-8/core collections: `String`, `Vec<String>`, `BTreeMap<String, String>`.
- `crates/crunch-shell/src/adapter.rs` keeps std-only translation work:
  - `split_paths(...)`
  - `PathBuf` → `String`
  - non-UTF-8 rejection via `ShellError::NonUtf8Path`
  - `ExecTarget` reconstruction after the core call
- `src/shell_cmd.rs` keeps shell-side effects outside the core:
  - reads `.crunch-shell.json`
  - snapshots host env via `std::env::vars()`
  - validates `--with` paths on disk
  - executes hook subprocesses and final shell/command exec

## Required evidence summary

Positive path ordering stays green. Negative non-UTF-8 input is rejected in the adapter. Inspection shows no `PathBuf` or `OsString` types crossing the `crunch-shell-core` public boundary, and `src/shell_cmd.rs` keeps host env snapshot, sidecar loading, hook execution, and final exec in the std shell.
