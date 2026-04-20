# NickelEvaluator Trait

## Overview

The seam between `crunch-system` (pure pipeline logic) and `crunch-eval`
(Nickel runtime). Defines the trait contract, value representation,
resource options, and import sandboxing.

## Requirements

### TRAIT-1: Trait definition

The `NickelEvaluator` trait MUST be defined in the `crunch-system` crate
with these methods:

- `evaluate_file(&self, path: &Path, opts: &EvalOptions) -> Result<NickelValue, EvalError>`
  — evaluate a `.ncl` file to a value. Used by the pipeline wiring step
  to produce `NickelValue` from raw module files before pure validation.
- `merge(&self, base: &NickelValue, overlay: &NickelValue, opts: &EvalOptions) -> Result<NickelValue, EvalError>`
  — merge two Nickel values with contract enforcement.
- `call(&self, function: &NickelValue, args: &NickelValue, opts: &EvalOptions) -> Result<NickelValue, EvalError>`
  — call a Nickel function with arguments.
- `get_field(&self, value: &NickelValue, key: &str) -> Result<NickelValue, EvalError>`
  — extract a named field from a Nickel record value. Returns
  `EvalError::NickelError` if the value is not a record or the field
  is missing.
- `is_function(&self, value: &NickelValue) -> Result<bool, EvalError>`
  — test whether a Nickel value is a function. Used by the loader to
  validate that `impl` is callable (LOADER-2).
- `to_json(&self, value: &NickelValue) -> Result<serde_json::Value, EvalError>`
  — serialize a Nickel value to JSON. Used by the evaluator to produce
  `EvaluatedFragment.data` before crossing the Nickel boundary into
  the pure fragment collector.

The trait MUST be object-safe (`dyn NickelEvaluator`).

### TRAIT-2: NickelValue type

`NickelValue` is a type alias for `crunch_eval::Expr` (which wraps
`nickel_lang::Expr`, itself a wrapper around `nickel_lang_core::term::RichTerm`).
It is re-exported by `crunch-system`. It MUST implement `Clone`.
It is NOT `Send` or `Sync` — all evaluation MUST happen on a single
thread per evaluator instance.

`crunch-eval` does NOT depend on `crunch-system`. The concrete
`NickelEvaluator` implementation is constructed in `crunch-system`
by wrapping `crunch-eval` APIs.

The evaluator pipeline threads Nickel values on a single thread.
Cross-thread boundaries (e.g., spawning builds) serialize through
`EvaluatedFragment.data` (JSON) and `MergedConfig`, both `Send + Sync`.

### TRAIT-3: EvalOptions

`EvalOptions` MUST include:
- `timeout: Option<Duration>` — per-call wallclock timeout. Default 60s.
- `import_paths: Vec<PathBuf>` — allowed import resolution directories.

Memory cap is a SHOULD: if the Nickel runtime gains a memory limit API,
`EvalOptions` adds `memory_limit_bytes: Option<u64>`. Until then,
timeout is the primary resource bound.

### TRAIT-4: Import sandboxing

The evaluator implementation MUST restrict Nickel `import` resolution
to paths within the configured `import_paths`. Imports resolving outside
these directories MUST fail with an `EvalError::ImportDenied` variant
naming the offending path.

Default import paths: the module directory and crunch's stdlib directory.

### TRAIT-5: EvalError type

`EvalError` MUST be an enum with at minimum:
- `NickelError(String)` — Nickel blame/contract/parse errors, rendered
  to a human-readable string.
- `Timeout { module_name: String, elapsed: Duration }`.
- `ImportDenied { path: PathBuf, allowed: Vec<PathBuf> }`.

`EvalError` MUST implement `std::error::Error`, `Send`, and `Sync`.

### TRAIT-6: Pipeline wiring

The CLI surface (or pipeline entry point) MUST call
`evaluator.evaluate_file()` for each module file, then pass the
resulting `NickelValue` to the loader's pure validation function.
This explicit step bridges file I/O and pure validation:

```
file discovery (I/O) → evaluate_file (Nickel boundary)
  → validate_module (pure core) → ValidatedModule
```

### TRAIT-7: Threading model

Because `NickelValue` is `!Send`, the entire Nickel evaluation
pipeline (evaluate_file, merge, call, to_json) MUST run on a single
dedicated OS thread. The async pipeline communicates with this thread
via a typed channel (request/response). Timeout enforcement is the
caller's responsibility via `tokio::time::timeout` on the channel
receive side.

Only serialized results cross the thread boundary:
- `serde_json::Value` for successful fragment data.
- `String` for Nickel diagnostic/blame error messages.
- `ValidatedModule` MUST NOT cross thread boundaries (it holds a
  `NickelValue` handle). It is consumed on the evaluator thread.

The dedicated thread is created once per `crunch system` invocation
and destroyed after all modules are evaluated.
