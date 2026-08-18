# Project input retention roots implementation validation

Date: 2026-07-01

## Implementation summary

- Added `crunch-project-core::retention` as the pure no-std functional core for retention policy validation, BLAKE3 lock-entry/root identity, generation selection, root action planning, stale/missing/interrupted diagnostics, and retained-record planning.
- Added manifest schema fields: project-level `retention` default and per-input `retention` override.
- Added CLI shell persistence in `src/project_cmd.rs` for `.mantle/retention.json` and `.mantle/retention-roots/*.json`, with same-directory temporary-file commits and `.mantle/retention.json.tmp` treated as uncommitted during checks.
- Added `mantle show` retention status output for `pinned`, `missing-root`, `stale-root`, `unpinned`, and `gc-eligible` states.

## Validation commands

```text
nix develop -c cargo test -p crunch-project-core
result: ok. 138 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```text
nix develop -c cargo test -p crunch-project
result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```text
nix develop -c cargo test -p mantle --test project_cli
result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```text
nix develop -c cargo test -p mantle --test project_refresh_cli
result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```text
nix develop -c rustfmt --check <touched Rust files>
result: completed successfully
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
result: valid = true, issues = []
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-input-retention-roots --root .
result: verdict = PASS, valid = true, issues = []
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate design project-input-retention-roots --root .
result: verdict = PASS, valid = true, issues = []
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-input-retention-roots --root .
result: verdict = PASS, valid = true, issues = []
```

```text
post-checkbox rerun:
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-input-retention-roots --root .
result: tasks verdict = PASS, valid = true, issues = []
```

```text
post-cleanup focused rerun:
nix develop -c cargo test -p crunch-project-core retention
result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 125 filtered out

nix develop -c cargo test -p mantle --test project_cli
result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Blocked checks

```text
nix develop -c cargo check -p crunch-project-core --target wasm32-unknown-unknown
blocked: wasm32-unknown-unknown target is not installed in the dev shell; rustup is not available in `nix develop`.
```

```text
nix develop -c cargo fmt --check -p crunch-project-core -p crunch-project -p mantle
blocked: package-wide fmt check reports an unrelated pre-existing diff in tests/attest_cli.rs. Touched files passed `rustfmt --check` directly.
```
