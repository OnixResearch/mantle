# Design: Extract crunch-pipeline

## Context

The crunch binary (`src/main.rs`) mixes CLI concerns with pipeline
orchestration. The pipeline — eval, deserialize, convert, bridge registry,
open store, construct builder, run worker, collect results — is ~600 lines
of business logic that only runs through `fn main`. Two code paths exist
(`execute_builds` and `execute_builds_streaming`), partially duplicated.
`cmd_self_build` reimplements store/builder setup.

## Goals / Non-Goals

**Goals:**

- One place where store + builder + worker are assembled. No duplication
  between `cmd_build`, `cmd_self_build`, or future `cmd_check`.
- The pipeline crate is testable: given a `.ncl` path and a config, run
  the full pipeline in-process and assert on outcomes.
- `main.rs` under 350 lines after extraction.

**Non-Goals:**

- Changing the Worker, Builder, or Goal APIs. Those are fine.
- Moving bootstrap logic. It has its own fetch pipeline that doesn't share
  the standard convert->build path.
- Abstracting over build services (the `#[cfg(target_os = "linux")]` stays —
  bwrap is the only backend).

## Decisions

### 1. Pipeline entry point: `BuildConfig` + `build()`

**Choice:** A single `build()` async function taking a `BuildConfig` struct.

```rust
pub struct BuildConfig {
    pub file: PathBuf,
    pub import_paths: Vec<OsString>,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub verbose: bool,
    pub max_jobs: u32,
    pub substituter_url: Option<String>,
}

pub struct PipelineResult {
    pub outcomes: Vec<BuildOutcome>,
    pub failed: Vec<FailedGoal>,
}

pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error>;
```

**Rationale:** A config struct replaces the 9-arg `execute_builds_streaming`.
Adding fields (e.g., `dry_run: bool`) doesn't break callers. The result is
a data structure — the caller decides how to format/print it.

**Alternative:** Trait-based pipeline with pluggable stages. Rejected —
the stages are fixed (eval->convert->build) and there's no second
implementation. A trait would add indirection with no consumers.

### 2. FOD mismatch as data, not control flow

**Choice:** `PipelineResult.fod_mismatches: Vec<FodMismatch>` carries
mismatch info. The binary applies `--fix` rewriting.

**Rationale:** The pipeline shouldn't read/write the source `.ncl` file
for `--fix`. That's a CLI concern (the user asked for it via a flag).
Returning the mismatch as structured data lets the binary decide what
to do.

**Implementation:** `FodMismatch { name, expected_sri, actual_sri }` struct.
The binary's `--fix` handler calls `auto_fix_hash()` (stays in binary or
moves to a small utility module).

### 3. Log writing stays in binary

**Choice:** `write_log()` stays in `main.rs`. The pipeline returns logs
inside `BuildOutcome.log`.

**Rationale:** Log directory location (`$CRUNCH_LOG_DIR`, `state_dir()`)
is a CLI/environment concern. The pipeline shouldn't know about the
filesystem layout of logs. Outcomes already carry the log content.

### 4. Remove `execute_builds` (non-streaming fallback)

**Choice:** Delete the non-streaming `execute_builds` function. Keep only
the streaming path.

**Rationale:** It's `#[allow(dead_code)]` already. The streaming path
handles single-derivation sets correctly (just one channel message). Two
paths means two places to fix bugs. The fallback existed as a safety net
during the streaming migration — that's done.

### 5. Store/builder construction inside the pipeline

**Choice:** The pipeline opens `StoreHandle`, constructs `BubblewrapBuildService`
and `Builder` internally. The caller provides `BuildConfig` with directory
paths.

**Rationale:** These are implementation details of *how* the build runs.
The binary shouldn't wire service objects. `cmd_self_build` currently
duplicates this wiring — after extraction it calls `build()` with a
config struct.

**Alternative:** Pass pre-constructed services to the pipeline. Rejected
for v0 — there's only one service configuration (redb + bwrap + on-disk
blobs). When alternative backends exist, the config struct can grow an
enum.

### 6. Crate depends on crunch-eval, crunch-glue, crunch-build, crunch-store

**Choice:** crunch-pipeline depends on all four crates. The binary depends
only on crunch-pipeline (plus clap, tracing-subscriber, tempfile).

**Rationale:** The pipeline IS the integration of these four stages. Having
all four as direct deps is correct. The binary drops direct deps on the
inner crates (except for types re-exported through crunch-pipeline).

## Risks / Trade-offs

**Compile time** — One more crate in the workspace. Incremental builds
are unaffected (crate graph is deeper, not wider). Clean builds add ~2s
for the crate's own compilation.

**API surface** — `BuildConfig` becomes a public commitment. Fields can
be added (with defaults) but not removed without a breaking change. This
is acceptable — the config mirrors CLI flags that are already committed.
