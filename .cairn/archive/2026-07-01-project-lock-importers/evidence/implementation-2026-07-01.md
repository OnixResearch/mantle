# Implementation evidence — project lock importers — 2026-07-01

## Question

Does the implemented `project-lock-importers` slice provide safe no-mutate/apply behavior for supported Nixtamal pin facts while refusing unsupported semantics?

## Decision

Implemented a scoped importer that is safe to archive:

- pure no-std importer core in `crates/crunch-project-core/src/importer.rs`;
- std shell/parser in `src/pin_import.rs` for bounded Nixtamal JSON fixtures;
- `mantle import pins plan|apply` CLI wiring in `src/main.rs`;
- CLI fixture coverage in `tests/pin_import_cli.rs`;
- future-adapter seams for `flake`, `npins`, and `niv` that are blocker-only and do not import recursive composition semantics.

Unsupported project-input semantics remain blockers instead of claims. Production support for richer freshness/fetch/trust/non-Git VCS behavior belongs to the corresponding active changes.

## Baseline

Before importer changes, focused project CLI smoke passed:

```text
$ cargo test -p mantle --test project_cli project -- --nocapture
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.06s
```

## Focused implementation evidence

```text
$ cargo test -p crunch-project-core importer::
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out; finished in 0.00s
```

```text
$ cargo test -p mantle --bin mantle pin_import::
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1039 filtered out; finished in 0.00s
```

```text
$ cargo test -p mantle --test pin_import_cli -- --nocapture
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

```text
$ cargo test -p mantle --test project_cli project -- --nocapture
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.05s
```

After warning cleanup in `soundness.rs`, the full core crate passed:

```text
$ cargo test -p crunch-project-core
test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests crunch_project_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Post-archive validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- archive project-lock-importers --root . && CAIRN_ARCHIVE_DATE=2026-07-01 nix run path:/home/brittonr/git/cairn#cairn -- archive project-lock-importers --root . --execute && nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```

Final validation after appending this archived evidence:

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```

## Non-claims

This evidence does not prove network fetching, source availability, full project freshness semantics, trust-policy verification, or non-Git VCS importer support. Those remain scoped to separate active changes or deterministic blockers.
