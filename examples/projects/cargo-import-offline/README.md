# Review-first Cargo import

This example converts a dependency-free/path-dependency Cargo workspace into a reviewable Mantle project handoff. Import is intentionally non-mutating until `--apply` is selected.

## Inspect the workflow contract

```sh
mantle eval workflow.ncl
```

## Plan and apply in a scratch copy

```sh
example_root=$(pwd)
work=$(mktemp -d /tmp/mantle-cargo-import.XXXXXX)
cp -R "$example_root/workspace/." "$work/"
cd "$work"

mantle --json import cargo --plan > import-plan.json
test ! -e mantle-project.ncl
test ! -e .mantle/inputs.ncl

mantle --json import cargo --apply > import-apply.json
test -f mantle-project.ncl
test -f .mantle/inputs.ncl
mantle eval mantle-project.ncl
```

Inspect `selected_package`, `selected_binary`, `file_operations[].digest_blake3`, `source_inputs`, `blockers`, and `non_claims`. Apply writes only `mantle-project.ncl` and `.mantle/inputs.ncl` after the plan is blocker-free.

The generated inputs file deliberately contains failing placeholders for the package source, Rust toolchain, seed toolchain, and musl. Replace them with admitted derivations before `mantle build`; see `examples/projects/rust-workspace/mantle-project.ncl` for the separately validated offline build lane. Import success alone is not a successful package build.

## Negative paths

Continue in the same shell so `example_root` still points at this example directory. A workspace with multiple default packages is rejected until one is selected:

```sh
cd "$example_root/ambiguous-workspace"
mantle --json import cargo --plan
mantle --json import cargo --plan --package app-a
```

A missing lockfile fails before file generation:

```sh
broken=$(mktemp -d /tmp/mantle-cargo-import-broken.XXXXXX)
cp -R "$example_root/workspace/." "$broken/"
rm "$broken/Cargo.lock"
cd "$broken"
mantle --json import cargo --apply
test ! -e mantle-project.ncl
test ! -e .mantle/inputs.ncl
```

From the repository root, the focused rail also covers stale vendor checksums, ambient `CARGO_HOME` rejection, unsupported registry sources, and existing-file conflicts:

```sh
nix develop -c cargo test -p mantle --test cargo_import_cli
```

Cargo import does not claim network vendoring, full Cargo compatibility, cargo-free execution, compiler correctness, or package success.
