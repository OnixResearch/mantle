## Why

The repo now has a real stage0 -> stage1 -> stage2 self-hosting proof, but the
operator path around it is still rough.

Two problems are visible from the current tree:

- the proof lives behind an ignored `cargo test`, and it needs the full Rust /
  clang / pkg-config / openssl environment set up correctly before it will even
  compile; a bare `cargo test -p crunch --test self_hosting -- --list` already
  fails on this machine without `cc` on PATH
- the main specs still tell an older story in a few places: `build-pipeline`
  says parallel builds are future work, and `portability` still describes
  WASM / gRPC builders as if they were current runtime paths

That leaves contributors with no single blessed proof command and no single
spec story they can trust. We should fix both in one follow-up change.

## What Changes

- Add one checked-in proof entry point that sets the required build environment
  and runs the existing ignored self-hosting proof test
- Document that entry point as the default contributor workflow for proving
  self-hosting locally
- Update the bootstrap spec to require that checked-in proof workflow
- Update the build-pipeline spec to describe bounded concurrent builds as a
  shipped behavior, not future work
- Update the portability spec so it matches the current Linux-first build
  reality and treats WASM / remote builders as future work until code lands

## Capabilities

### New Capabilities
- `self-hosting-proof-workflow`: one checked-in, repeatable command path for
  running the stage0 -> stage1 -> stage2 proof with the required environment

### Modified Capabilities
- `bootstrap`: contributor proof workflow is explicit and reproducible
- `build-pipeline`: main spec reflects the current concurrent worker behavior
- `portability`: main spec matches the current Linux-only build support
- `documentation`: README and specs point at the same proof and runtime model

## Impact

- **Files**: a new repo-local proof helper, `README.md`, and the main specs in
  `openspec/specs/bootstrap/`, `openspec/specs/build-pipeline/`, and
  `openspec/specs/portability/`
- **APIs**: none required beyond the existing self-hosting proof test surface
- **Dependencies**: none new; reuse the current proof prerequisites
- **Testing**: run the checked-in proof entry point, plus `openspec validate`
  for the change
