# Verification Evidence: Extract crunch-pipeline

This file records current end-state evidence for the checked tasks in
`tasks.md`. It is not commit-by-commit history.

## Phase 1: Scaffold crate, move types

### Workspace and binary wiring

```toml
# Cargo.toml
[workspace]
members = [
  "crates/crunch-eval",
  "crates/crunch-glue",
  "crates/crunch-store",
  "crates/crunch-build",
  "crates/crunch-pipeline",
  ...
]

[dependencies]
crunch-pipeline = { path = "crates/crunch-pipeline" }
```

### Pipeline crate dependencies

```toml
# crates/crunch-pipeline/Cargo.toml
[dependencies]
crunch-build = { path = "../crunch-build" }
crunch-eval = { path = "../crunch-eval" }
crunch-glue = { path = "../crunch-glue" }
crunch-store = { path = "../crunch-store" }
nix-compat = { path = "../../vendor/nix-compat" }
snix-build = { path = "../../vendor/snix-build" }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1", features = ["rt", "sync"] }
tracing = "0.1"
```

### Public types and required fields

```text
$ nl -ba crates/crunch-pipeline/src/lib.rs | sed -n '10,35p'
    12  pub struct BuildConfig {
    13      pub file: PathBuf,
    14      pub import_paths: Vec<OsString>,
    15      pub output_dir: PathBuf,
    16      pub state_dir: PathBuf,
    17      pub store_dir: String,
    18      pub verbose: bool,
    19      pub max_jobs: u32,
    20      pub substituter_url: Option<String>,
    21  }
    24  pub struct PipelineResult {
    25      pub outcomes: Vec<BuildOutcome>,
    26      pub failed: Vec<FailedGoal>,
    27      pub fod_mismatches: Vec<FodMismatch>,
    28      pub root_labels: HashMap<String, String>,
    29  }
    32  pub struct FodMismatch {
    33      pub name: String,
    34      pub expected_sri: String,
    35      pub actual_sri: String,
```

### Compile evidence

```text
$ cargo check -p crunch-pipeline
Checking crunch-pipeline v0.1.0 (/home/brittonr/git/crunch/crunch/crates/crunch-pipeline)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.34s
```

## Phase 2: Extract pipeline functions

### Extracted helpers and pipeline entry point

```text
$ rg --line-number 'pub fn resolve_max_jobs|pub fn deserialize_derivations_from_json|pub fn parse_fod_mismatch_error|pub async fn build|fn convert_all' crates/crunch-pipeline/src/lib.rs
52:pub fn resolve_max_jobs(user: Option<u32>) -> u32 {
62:pub fn deserialize_derivations_from_json(
101:pub fn parse_fod_mismatch_error(err: &str) -> Option<FodMismatch> {
112:pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error> {
220:fn convert_all(
```

### `build()` owns store/builder/worker setup and result collection

```text
$ nl -ba crates/crunch-pipeline/src/lib.rs | sed -n '114,184p'
   114  pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error> {
   117      let json_str = crunch_eval::evaluate_to_json(&config.file, &config.import_paths)
   122      let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
   145      let mut builder = Builder::with_state_dir(
   157      let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
   161      let mut known_paths = DerivationRegistry::new(&config.store_dir);
   162      let mut worker = Worker::new(config.max_jobs);
   163      let worker_run = worker.run_streaming(&mut builder, &mut known_paths, &mut rx).await;
   165      let root_drv_paths =
   171      let mut worker_result = match worker_run {
   172          Ok(result) => result,
   173          Err(err) => return Err(Error::Build(format!("{err}"))),
   174      };
   175      normalize_failed_goal_keys(&mut worker_result.failed, &config.store_dir);
   176      let root_labels = build_root_labels(&root_drv_paths, &config.store_dir);
   177      let fod_mismatches = collect_fod_mismatches(&worker_result.failed);
   179      Ok(PipelineResult {
   180          outcomes: worker_result.outcomes,
   181          failed: worker_result.failed,
   182          fod_mismatches,
   183          root_labels,
   184      })
```

### `convert_all()` owns the convert loop and registry-bridge payloads

```text
$ nl -ba crates/crunch-pipeline/src/lib.rs | sed -n '220,246p'
   220  fn convert_all(
   225      let mut cache = ConversionCache::new(store_dir);
   228      for (label, drv) in &derivations {
   229          let (drv_path, _nix_drv) = crunch_glue::convert(drv, &mut cache)
   232          let new_entries = cache.drain_pending();
   235          tx.blocking_send(EvalMessage {
   236              label: label.clone(),
   237              drv_path: drv_path.clone(),
   238              new_entries,
   239          })
   242          drv_paths.push((label.clone(), drv_path));
   245      drop(tx);
   246      Ok(drv_paths)
```

### Moved helper removal evidence in the binary crate

```text
$ rg -n '^pub fn deserialize_derivations_from_json|^pub fn resolve_max_jobs|^pub fn parse_fod_mismatch_error' src --glob '*.rs'
(no helper definitions in src/*.rs)

$ rg -n 'execute_builds' src --glob '*.rs'
(no matches in src/*.rs)
```

### Compile evidence

```text
$ cargo check -p crunch-pipeline
Checking crunch-pipeline v0.1.0 (/home/brittonr/git/crunch/crunch/crates/crunch-pipeline)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.34s
```

## Phase 3: Rewire main.rs

### `cmd_build` constructs `BuildConfig`, then `run_build` passes `&BuildConfig`

This resolves the signature/call-site ambiguity: `cmd_build` creates an owned
`BuildConfig`, then passes `&config` into `run_build`, and `run_build` calls
`crunch_pipeline::build(config)` where its own `config` parameter is already a
reference.

```text
$ nl -ba src/build_cmd.rs | sed -n '9,40p'
     9  pub fn cmd_build(
    20      let config = BuildConfig {
    31      let result = run_build(&config)?;
    35  pub fn run_build(config: &BuildConfig) -> Result<PipelineResult, RunError> {
    38      rt.block_on(crunch_pipeline::build(config)).map_err(Into::into)
```

### `fix.rs` owns FOD mismatch handling in the binary

```text
$ rg -n '^pub fn handle_fod_mismatch|^pub fn auto_fix_hash' src --glob '*.rs'
src/fix.rs:9:pub fn handle_fod_mismatch(
src/fix.rs:52:pub fn auto_fix_hash(file: &Path, old_hash: &str, new_hash: &str) -> Result<(), String> {
```

### CLI concerns stay in the binary

```text
$ nl -ba src/build_cmd.rs | sed -n '150,205p'
   155  pub fn write_log(
   174  pub fn state_dir() -> PathBuf {
   188  pub fn log_dir() -> PathBuf {
   194  pub fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
```

### `cmd_self_build` delegates the actual build through pipeline config + `run_build`

```text
$ nl -ba src/self_build.rs | sed -n '412,480p'
   412  pub fn cmd_self_build(
   453      let config = crunch_pipeline::BuildConfig {
   468      let result = run_build(&config)?;
   469      report_build_result(&config, &result, false)?;
```

### `main.rs` size

```text
$ wc -l < src/main.rs
320
```

### Dependency audit evidence

The audit found no more removable normal dependencies in the binary crate.
Every remaining direct dependency still has a live `src/` call site.

```text
$ rg --line-number 'crunch_pipeline|crunch_eval|crunch_glue|crunch_build|crunch_store|snix_build|snix_castore|snix_store|tokio|tracing_subscriber|tracing::|clap|data_encoding|tempfile|blake3' src --glob '*.rs'
src/main.rs:12:use clap::{Parser, Subcommand};
src/main.rs:181:    tracing_subscriber::fmt()
src/main.rs:214:            let json = crunch_eval::evaluate_to_json(&file, &import_paths)
src/main.rs:228:            let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
src/build_cmd.rs:4:use crunch_pipeline::{BuildConfig, PipelineResult, drv_key_for, label_for_key, parse_drv_key};
src/build_cmd.rs:36:    let rt = tokio::runtime::Runtime::new()
src/build_cmd.rs:195:    let stdlib_dir = crunch_eval::stdlib::stdlib_import_path()
src/bootstrap.rs:143:fn make_fetch_derivation(seed: &FetchSeed) -> crunch_glue::CrunchDerivation {
src/bootstrap.rs:248:        use snix_castore::blobservice::ObjectStoreBlobService;
src/bootstrap.rs:272:        use snix_build::buildservice::BubblewrapBuildService;
src/bootstrap.rs:289:        let mut builder = crunch_build::Builder::with_state_dir(
src/store_cmd.rs:43:    let entries = crunch_store::store_list(svc)
src/store_cmd.rs:5:    use snix_store::pathinfoservice::{RedbPathInfoService, RedbPathInfoServiceConfig};
src/self_build.rs:91:    let digest_bytes = data_encoding::HEXLOWER.decode(fingerprint.as_bytes())
src/self_build.rs:368:    let mut hasher = blake3::Hasher::new();
src/self_build.rs:440:    let tmp_dir = tempfile::tempdir()

$ rg --line-number 'nix_compat' src
src/bootstrap.rs:305:        let root_paths: Vec<nix_compat::store_path::StorePath<String>> =
src/self_build.rs:93:    let store_hash = nix_compat::nixbase32::encode(&digest_bytes[..20]);
src/build_cmd.rs:5:use nix_compat::store_path::StorePath;
src/fix.rs:4:use nix_compat::store_path::StorePath;
src/main.rs:47:    nix_compat: bool,
src/main.rs:268:                nix_compat::store_path::STORE_DIR,
src/main.rs:279:    if args.nix_compat {
```

### Compile evidence

```text
$ cargo check -p crunch
Checking crunch v0.1.0 (/home/brittonr/git/crunch/crunch)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.07s
```

## Phase 4: Tests

### Pipeline unit and integration tests

```text
$ cargo test -p crunch-pipeline
Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_pipeline-8812cc8c4d9916cd)

running 11 tests
test tests::parse_fod_mismatch_invalid ... ok
test tests::parse_fod_mismatch_valid ... ok
test tests::parse_fod_mismatch_edge_case_preserves_trailing_context ... ok
test tests::parse_fod_mismatch_strips_drv_suffix ... ok
test tests::parse_drv_key_round_trip ... ok
test tests::deserialize_package_set ... ok
test tests::deserialize_invalid_json_errors ... ok
test tests::normalize_failed_goal_keys_rewrites_nix_store_keys ... ok
test tests::resolve_max_jobs_clamps_user_value ... ok
test tests::deserialize_single_derivation ... ok
test tests::resolve_max_jobs_default_in_range ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

Running tests/integration_build.rs (/home/brittonr/.cargo-target/debug/deps/integration_build-169b945e0ae32323)

running 2 tests
test pipeline_builds_trivial_derivation_end_to_end ... ok
test pipeline_reports_fod_mismatch_without_aborting_other_roots ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

### Real FOD mismatch integration evidence

The new integration test uses two `file://` fetchurl roots in one `.ncl` file:
one with the correct sha256 and one with an intentionally wrong sha256. It uses
the same `can_build()` skip as the existing end-to-end test, including the
`bwrap --version` probe, so hosts without `bwrap` keep the old skip behavior.
When the test runs, the pipeline returns `Ok(PipelineResult)` with one
successful outcome, one failed root, and one parsed `fod_mismatch` entry, which
exercises the spec path that a FOD mismatch is reported as data without
aborting sibling roots.

```text
$ nl -ba crates/crunch-pipeline/tests/integration_build.rs | sed -n '12,18p'
    12  fn can_build() -> bool {
    13      Path::new("/nix/store").exists()
    14          && std::process::Command::new("bwrap")
    15              .arg("--version")
    16              .output()
    17              .is_ok_and(|output| output.status.success())
    18  }

$ cargo test -p crunch-pipeline pipeline_reports_fod_mismatch_without_aborting_other_roots -- --exact
...
running 1 test
test pipeline_reports_fod_mismatch_without_aborting_other_roots ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

### Vendor regression checks mentioned in this change

```text
$ cargo test -p fuse-backend-rs transport::fusedev::linux_session::tests::test_new_channel
running 1 test
test transport::fusedev::linux_session::tests::test_new_channel ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.00s

$ cargo test -p snix-castore --doc
Doc-tests snix_castore
running 3 tests
test vendor/snix-castore/src/composition.rs - composition (line 16) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 84) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 52) ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Workspace regression check

This was re-run after adding the `can_build()` skip to the new FOD integration
test.

```text
$ cargo test --workspace >/tmp/crunch-workspace-test.log && echo WORKSPACE_EXIT=0 && tail -n 30 /tmp/crunch-workspace-test.log
WORKSPACE_EXIT=0
...
running 3 tests
test vendor/snix-castore/src/composition.rs - composition (line 16) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 84) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 52) ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.93s; merged doctests compilation took 0.91s

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 1 test
test vendor/snix-tracing/src/lib.rs - TracingBuilder::build_with_additional (line 244) ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
