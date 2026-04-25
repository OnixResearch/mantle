# Design: `crunch run`

## Context

`src/main.rs` already has project-aware build helpers and a partial `run`
command that builds a `crunch.ncl` package-project selector and executes the
first file or symlink under the selected output's `bin/` directory. The
OpenSpec change closes the command into a documented operator surface: package
project defaults, bare package names, project selectors, direct single-derivation
file targets, binary selection, argument passthrough, and deterministic failure
messages. This command uses the existing package/build project file
`crunch.ncl`; it does not consume the dependency-management manifest
`crunch-project.ncl`.

## Decisions

### 1. Keep `run` as a thin CLI shell over existing build plumbing

**Choice:** Reuse `build_cmd::run_build` through the existing
`BuildConfig` path. `run` should not have a separate evaluator or builder.

**Rationale:** Build, substitution, signing, store-prefix handling, and
hermeticity are already centralized in the build pipeline. `run` is command
orchestration: resolve a target, build it, find one executable, then run it.

**Implementation:** Keep target resolution, result-output selection, binary
selection, and process launching as small pure-ish helpers with one imperative
shell that performs build and `Command` execution.

### 1a. Select one output from the selected derivation

**Choice:** When the selected derivation has multiple outputs, run uses the
`out` output if present. If `out` is absent, run selects the first output name in
lexicographic order.

**Rationale:** This mirrors the existing build-report preference and keeps
multi-output run behavior deterministic without inventing a new package metadata
field.

**Implementation:** Return the selected `PathInfo` together with the host output
path instead of only returning the first host path. Use a `BTreeMap`/sorted
output-name order for fallback.

### 2. Resolve both project and file targets

**Choice:** `crunch run` with no target uses the current `crunch.ncl` package
project's default package. `crunch run hello` resolves `packages.hello` from
that project. `crunch run .#name` and nested selectors such as `crunch run
.#packages.hello` resolve project selectors. `crunch run path/to/file.ncl`
builds that file directly only when it evaluates directly to exactly one derivation, not a record.

**Rationale:** Project selectors and bare project package names cover the
existing project workflow, while explicit files preserve the ad-hoc `nix run
./expr` style proposed by the change. A non-selector target that starts with
`./`, `../`, or `/`, or ends in `.ncl`, is a file target; other bare words stay
project package names even when a same-named filesystem entry exists. Global
curated-package lookup is deferred until the package-set change defines that
registry.

**Implementation:** Extend the existing `name_to_build_target` dispatch so
`BuildTarget::File` flows through a direct build path instead of project
resolution. Direct file builds use the same import-path construction as
`crunch build`, then require the evaluated target and pipeline result to contain
exactly one top-level derivation before binary selection and reject record-valued files, including singleton records. Bare names continue to map to `ProjectTarget::Attribute`.

### 3. Binary discovery is explicit when requested, deterministic otherwise

**Choice:** Add `--bin <name>`. When supplied, `run` executes exactly
`$out/bin/<name>` and errors if it is missing, is a directory, or is not
executable. Without `--bin`, `run` ignores directories, sorts file/symlink
entries under `$out/bin` by file name, and runs the first candidate.

**Rationale:** Multi-binary packages need deterministic operator choice.
Alphabetical fallback keeps the current simple case working and avoids host
filesystem iteration order leaking into behavior.

**Implementation:** Split executable selection into a helper returning a path
or a structured `RunError`. Filter candidate entries to files or symlinks before
fallback selection, reject empty/path-like `--bin` names before joining with
`$out/bin`, skip non-executable regular files in fallback mode, validate execute
bits on Unix target metadata, and keep sorting by file name before choosing the
fallback. Explicit `--bin` rejects broken symlinks, symlinks to directories, and
symlinks to non-executable targets. Fallback mode skips those invalid symlinks
and fails only when no valid executable file or symlink candidate remains.

### 4. Argument passthrough and exit status match the executed program

**Choice:** Arguments after `--` are passed unchanged to the selected binary.
`crunch run` exits with the executed program's exit status.

**Rationale:** `run` is an execution wrapper, not an output rewriter. Scripts
and callers should be able to rely on ordinary process semantics.

**Implementation:** Preserve the existing `#[arg(last = true)]` argument
capture. Continue launching through `std::process::Command`, inherit the
parent's stdio/environment/current directory defaults, and exit with the child's
status code after the child returns. If the child terminates without an exit
code, map that status to exit code `1`.

### 5. Run uses practical hermeticity for this change

**Choice:** Keep `crunch run` in `HermeticityMode::Practical` and do not add a
`--strict-hermetic` flag in this change.

**Rationale:** The main CLI spec currently requires strict-hermetic selection at
minimum for `build` and `self-build`. `run` already acts as a convenience wrapper
for local execution, and strict run policy needs a separate operator decision
because it affects interactive executable behavior.

**Implementation:** Construct run `BuildConfig` with practical hermeticity and
let clap reject `crunch run --strict-hermetic` as an unknown flag.

## Verification

- OpenSpec validation: `openspec validate crunch-run --strict`.
- Unit tests for binary selection cover explicit `--bin`, sorted fallback,
  missing `bin/`, empty `bin/`, directory entries under `bin/`, and
  non-executable regular files.
- Resolver tests cover selector-before-filesystem precedence, bare package
  versus same-named path ambiguity, simple and nested selectors, explicit `.ncl`
  file targets, record-valued file rejection, missing default-package selection,
  and missing project/file/selector failures; diagnostics must mention
  `crunch.ncl` for missing package projects and the rejected target for
  unrunnable targets.
- CLI integration tests cover project default, bare project package name,
  project selector, explicit file target, selected output (`out` before sorted
  fallback), argument passthrough, child exit status/no-code mapping, missing
  binary failures, path-like `--bin` rejection, symlink validation, practical
  hermeticity/no strict flag, profile non-mutation, and caller-supplied
  `--import-path`, `--jobs`, `--no-substitute`, `--signing-key`,
  `--trust-unsigned`, `--store`, `--state-dir`, `--store-prefix`, and
  `--nix-compat` forwarding where each flag can be observed deterministically.
- README/help documentation is updated so the top-level shipped command summary
  covers `run` target forms, `--bin`, and `--` passthrough, or links to focused
  CLI docs with those details.

## Risks

- Direct file targets can be ambiguous with package names. The resolver treats
  only explicit path syntax (`./`, `../`, `/`, or `.ncl` suffix) as file syntax,
  while bare names remain package selectors even when a same-named path exists.
- `Command::status` does not replace the current process image. This keeps the
  implementation portable for tests, but wrappers still observe the child exit
  code.
