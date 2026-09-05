# Package test fixtures

## Scope

The package builds the production CLI with the release profile. Its ordinary
integration tests continue to use that release binary.

Evaluator fault injection is different. `src/evaluator_budget.rs` intentionally
excludes those hooks from release builds. The package check builds a separate
debug fixture executable under `TMPDIR`. Nextest uses the same preparation.
Only fault-injection cases use `MANTLE_TEST_EVALUATOR_BINARY`.
The fixture executable is not an installed package output.

The release-only test `release_binary_ignores_fault_injection_environment`
requires the ordinary release binary to ignore panic, cancellation, and reap
failure fixture variables. No production guard changes for these tests.
A missing fixture executable fails a release test rather than skipping it.

This is test and packaging maintenance, not an accepted-spec change.
No production CLI, supervisor, frontend admission, or release authority changes.
Fixture evidence does not prove every production failure mode or resource bound.

## Test tools

The package, nextest, and development environments declare Python for the
OpenSSL patch test. This declaration does not add a production Python capability.

Ptrace fixtures use `MANTLE_TEST_SCRIPT_SHELL`. When that variable is absent,
standalone native tests use `/bin/sh`. An invalid explicit value fails without
fallback. Parent and descendant commands use the same canonical executable.
The descendant fixture includes spaces and quotes in its selected filename.
Digest denial, teardown, concurrency, and output assertions remain required.

The frontend socket fixture requires the current Bounded Tree
`PlanRejected` / `UnsupportedKind` diagnostic for a directory member.
It separately checks the existing unsupported-type diagnostic for a socket root.
Both rejections must occur before store state exists. An arbitrary error is not
a passing result.

## Commands

Run the complete native package check:

```sh
nix build .#default --no-link -L
```

For focused checks, enter the package build environment:

```sh
nix develop .#packages.x86_64-linux.default
export CARGO_TARGET_DIR="$PWD/target"
cargo test --locked -p mantle --bin crunch protected_exec_ptrace::linux::tests
cargo test --locked -p mantle --bin crunch oci_projection_shell::tests
cargo test --locked -p mantle --test evaluator_budget_cli
```

For native x86-64 Linux release integration tests, provide the separate fixture:

```sh
cargo build --locked --profile dev --bin mantle --target x86_64-unknown-linux-gnu
export MANTLE_TEST_EVALUATOR_BINARY="$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/debug/mantle"
cargo test --locked --release -p mantle --test evaluator_budget_cli
```

Do not enable evaluator fault injection in the installed release CLI to make
the fixture tests pass. Do not replace negative assertions with success or skips.
A package test result does not admit a downstream runtime or authorize promotion.
