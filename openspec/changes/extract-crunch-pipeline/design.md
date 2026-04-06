## Context

main.rs is 1335 lines. It contains the full eval→build→realize pipeline,
service construction, FOD handling, log management, and store queries.
The binary crate's lib.rs exports nothing useful.

## Goals / Non-Goals

**Goals:** Make the pipeline embeddable as a library. Reduce main.rs to
argument parsing and output formatting. Enable testing the pipeline
without a CLI.

**Non-Goals:** Change the pipeline's behavior. Add new commands. Change
the Worker/Goal architecture.

## Decisions

### 1. crunch-pipeline as a new crate

**Choice:** New crate at `crates/crunch-pipeline/`.

**Rationale:** The pipeline is the integration layer between eval, glue,
build, and store. It doesn't belong in any of those crates. It's the
"main logic" extracted from main.rs.

**Alternative:** Put it in the binary crate's lib.rs. Rejected because
lib.rs in a binary crate is awkward — it's not independently publishable
or testable, and `cargo doc` doesn't treat it the same as a library crate.

### 2. Structured result types

**Choice:** `BuildResult` struct with per-derivation outcomes, logs,
and failures. Not just a list of paths.

**Rationale:** Library consumers need structured data. CLI consumers
need formatted output. The CLI formats the struct; the library returns it.

### 3. Pipeline owns state directory resolution

**Choice:** `state_dir()`, `log_dir()` move to crunch-pipeline (or
crunch-store). The binary crate doesn't know where state lives.

**Rationale:** Library consumers may want to override state location.
The pipeline function takes an optional state dir in its config.

### 4. Ordering: extract-crunch-store first

**Choice:** This change depends on extract-crunch-store landing first.

**Rationale:** The pipeline needs to construct a StoreHandle and pass
it to Builder. Without crunch-store, the pipeline would duplicate the
service construction logic from main.rs.

## Risks / Trade-offs

**[API stability]** The pipeline API becomes a public interface. Changes
to it affect library consumers. Mitigated by keeping the API minimal
(build, eval, bootstrap) and using option structs for configuration.

**[Integration test migration]** Tests in `tests/` that call the binary
via assert_cmd still work. Tests that exercise the pipeline directly
move to crunch-pipeline.
