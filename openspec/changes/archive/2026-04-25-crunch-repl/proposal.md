## Why

There is no interactive way to explore crunch's Nickel environment. When writing derivations or debugging evaluation, you currently write a `.ncl` file, run `crunch eval`, check the output, edit, repeat. A REPL with crunch's stdlib pre-loaded would make this feedback loop immediate.

Nickel itself has a REPL (`nickel repl`), but it doesn't load crunch's stdlib (`lib/lib.ncl`, `lib/fetch.ncl`, `lib/derivation.ncl`, etc.) or know about crunch's import paths. The gap is a crunch-aware REPL entry point.

## What Changes

- **`crunch repl`**: Launches a Nickel REPL session with crunch's stdlib import paths pre-configured. `import "lib.ncl"`, `import "fetch.ncl"`, etc. work out of the box.
- **Stdlib pre-import**: Optionally pre-import `lib.ncl` into scope so `crunch.fetchurl`, `crunch.mkDerivation`, etc. are immediately available without an explicit import.
- **Project context**: When run inside a project directory (with `crunch-project.ncl`), the REPL also loads the project's inputs and import paths so you can inspect resolved dependencies interactively.
- **Evaluation hooks**: `:build <expr>` or similar REPL command that takes the current expression, runs it through the build pipeline, and shows the result — bridging interactive exploration with actual builds.

## Capabilities

### New Capabilities
- `repl`: Interactive Nickel session with crunch stdlib
- `repl-project`: Project-aware REPL with inputs in scope
- `repl-build`: In-REPL build trigger for quick iteration

### Modified Capabilities
- `eval`: No changes — REPL is a separate entry point using the same evaluator

## Impact

- **Files**: New `src/repl_cmd.rs`, `src/main.rs` (subcommand), integration with `crates/crunch-eval/`
- **APIs**: New CLI subcommand. May need Nickel REPL API access from `nickel-lang-core`.
- **Dependencies**: `nickel-lang-core` REPL internals (already vendored in the Nickel workspace)
- **Testing**: Smoke test that REPL starts, can evaluate `1 + 1`, and can import crunch stdlib
