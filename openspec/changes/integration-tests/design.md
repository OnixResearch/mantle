## Context

crunch is a single binary with three subcommands. The binary links all four
crates (crunch-eval, crunch-glue, crunch-build, plus the CLI in `src/`).
Integration tests exercise the compiled binary end-to-end.

The `crunch build` subcommand requires Linux with bwrap. The `crunch eval`
and `crunch bootstrap` subcommands are platform-independent. Tests must
handle this gracefully.

## Goals / Non-Goals

**Goals:** Test the binary's observable behavior — stdout, stderr, exit code,
filesystem side effects. Cover the seams between crates that unit tests miss.

**Non-Goals:** Don't test Nickel language features. Don't test bwrap sandbox
internals. Don't benchmark build performance.

## Decisions

### 1. Use `assert_cmd` for binary invocation

**Choice:** Use `assert_cmd::Command` to build and invoke the `crunch`
binary, assert on stdout/stderr/exit code.

**Rationale:** Standard Rust pattern for CLI integration tests. Handles
binary path resolution, output capture, and assertion helpers.

**Alternative:** Shell scripts. Rejected — harder to maintain, less precise
assertions, no compile-time checking of fixture paths.

### 2. Gate build tests on Linux

**Choice:** Use `#[cfg(target_os = "linux")]` for tests that invoke
`crunch build`. Run eval and bootstrap tests on all platforms.

**Rationale:** bwrap only works on Linux. CI and most dev machines are
Linux, so the tests run where it matters. Non-Linux gets eval/bootstrap
coverage.

### 3. Use a temp `--store` directory for build tests

**Choice:** Create a temp directory, pass `--store /tmp/crunch-test-store-xxx`,
and run builds against it.

**Rationale:** Tests must not write to `/nix/store`. A temp store
isolates test artifacts and cleans up automatically.

**Constraint:** The builder scripts need real tools (bash, coreutils) from
`/nix/store`. The seed will reference real paths. So the test store is only
for *output* paths — input paths still come from the real Nix store.

### 4. Fixture `.ncl` files in `tests/fixtures/`

**Choice:** Put test Nickel files in `tests/fixtures/` alongside the
integration test source.

**Rationale:** Keeps test inputs versioned, easy to reference from
`assert_cmd`, clear separation from production lib/ files.

### 5. Multi-derivation test

**Choice:** Test a `.ncl` file that returns a record-of-records (no `name`
at top level) and verify that all derivations are evaluated.

**Rationale:** The single-vs-multi detection in `main.rs` checks for
`name` field presence. This is a heuristic with no tests. A fixture that
exercises both paths prevents regressions.

## Risks / Trade-offs

**[Nix store dependency]** → Build tests assume `/nix/store` has bash and
coreutils. This is true on NixOS and Nix-installed systems but not
everywhere. Accept this — crunch fundamentally depends on Nix store paths.

**[Test speed]** → Build tests invoke bwrap, which takes ~100ms minimum.
Keep the number of build tests small (3–5). Eval tests are fast (~50ms each).

**[Flaky builds]** → Sandboxed builds can fail for system reasons (no
unprivileged user namespaces, missing /bin/sh). Tests should have clear
skip conditions.
