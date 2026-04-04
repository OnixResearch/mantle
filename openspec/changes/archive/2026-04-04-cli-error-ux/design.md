## Context

`main.rs` handles three commands (build, eval, bootstrap) with a flat
`RunError` enum mapped to exit codes. Build failures wrap the raw error
string. Bootstrap shells out to nix-build/nix with no abstraction layer.

## Goals / Non-Goals

**Goals:** Make error output actionable. Test the CLI logic without
requiring a real Nix installation or bwrap.

**Non-Goals:** Don't restructure the CLI framework (clap is fine). Don't
add subcommands.

## Decisions

### 1. Extract error formatting into a testable module

**Choice:** Move `RunError` and its Display/exit-code logic into
`src/errors.rs`. The formatting is pure — takes an error, returns a
string and exit code. Testable without IO.

**Rationale:** `main()` becomes a thin shell: parse args, call run(),
format error. Each piece is testable.

### 2. Build failure message extraction

**Choice:** Parse the bwrap error string to extract the builder's stderr.
The `BubblewrapBuildService` includes sandbox stdout/stderr in its error.
Split on known delimiters to separate sandbox machinery from builder output.

**Rationale:** Users care about their build script's errors, not bwrap's
namespace setup messages.

### 3. Bootstrap testability via command injection

**Choice:** Extract the nix-build/nix-build logic into a function that
takes a `Fn(pkg) -> Result<String>` resolver. Tests pass a mock resolver.
Production passes the real nix-build/nix-build calls.

**Rationale:** No filesystem or nix installation needed for tests.

### 4. `--json` errors

**Choice:** Add `--json` global flag. When set, errors are emitted as
`{"error": "...", "code": N, "kind": "eval|build|internal"}`.

**Rationale:** Enables editor/CI integration without string parsing.

## Risks / Trade-offs

**[bwrap error format]** → Parsing bwrap output is fragile. If the format
changes, we fall back to the raw message. Acceptable.
