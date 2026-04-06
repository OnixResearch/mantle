# Architecture Specification — Delta

## ADDED Requirements

### Requirement: crunch-pipeline crate

The workspace MUST contain a `crunch-pipeline` crate that owns the
eval→build→realize pipeline. The binary crate (`crunch`) MUST be a
thin CLI shell that parses arguments and calls into crunch-pipeline.

| Crate | Role |
|---|---|
| `crunch` (binary) | CLI argument parsing, output formatting. Under 200 lines. |
| `crunch-pipeline` | Pipeline orchestration: eval, convert, build, collect results |
| `crunch-eval` | Nickel evaluation |
| `crunch-glue` | Nickel→Derivation conversion, ConversionCache |
| `crunch-build` | Goal scheduler, Worker, build dispatch |
| `crunch-store` | Store services, caching, realization |
| vendored crates | Data layer (nix-compat, snix-build, snix-castore, snix-store) |

#### Scenario: Library embedding

- GIVEN a Rust program that depends on `crunch-pipeline`
- WHEN it calls `pipeline::build(file, opts)`
- THEN a build executes and returns structured results
- AND no CLI parsing or terminal output occurs

#### Scenario: Binary crate is thin

- GIVEN the `src/main.rs` file
- WHEN its line count is measured
- THEN it is under 200 lines

### Requirement: Pipeline entry points

`crunch-pipeline` MUST provide at minimum:

```rust
pub async fn build(opts: BuildOpts) -> Result<BuildResult, Error>;
pub fn eval(opts: EvalOpts) -> Result<String, Error>;
pub async fn bootstrap(opts: BootstrapOpts) -> Result<(), Error>;
```

Each function is self-contained: it constructs services, runs the
pipeline, and returns structured results. The caller does not wire
services together.

#### Scenario: Build returns structured results

- GIVEN a valid .ncl file
- WHEN `pipeline::build(opts)` is called
- THEN `BuildResult` contains output paths, per-package outcomes,
  logs, and failure details

#### Scenario: Eval returns JSON

- GIVEN a valid .ncl file
- WHEN `pipeline::eval(opts)` is called
- THEN a JSON string is returned (same as `crunch eval` output)

### Requirement: Derivation deserialization in pipeline

The logic for detecting single-derivation vs package-set JSON and
deserializing into `CrunchDerivation` MUST live in `crunch-pipeline`,
not in the binary crate.

#### Scenario: Package set handling

- GIVEN JSON with multiple top-level fields (no "name" key)
- WHEN `pipeline::build()` processes it
- THEN each field is treated as a separate derivation

### Requirement: Log management in pipeline

Build log persistence (`write_log`), retrieval (`cmd_log`), and
state directory resolution (`state_dir`, `log_dir`) MUST live in
`crunch-pipeline` or `crunch-store`, not in the binary crate.

#### Scenario: Retrieve logs from library

- GIVEN a completed build
- WHEN `pipeline::logs(query)` is called from library code
- THEN matching build logs are returned

### Requirement: FOD error handling in pipeline

FOD hash mismatch detection, `--fix` source rewriting, and the
re-run error message MUST live in `crunch-pipeline`, not in the
binary crate.

## MODIFIED Requirements

### Requirement: Pipeline stages (modified)

The architecture spec's pipeline stages are unchanged, but the
wiring MUST live in `crunch-pipeline`:

1. Nickel source → evaluated record (crunch-eval)
2. Evaluated record → Derivation structs (crunch-glue)
3. Derivation → stream to Worker (crunch-pipeline)
4. Worker → BuildService dispatch (crunch-build)
5. BuildResult → PathInfo persistence (crunch-store)

The binary crate MUST NOT contain any of steps 1–5.
