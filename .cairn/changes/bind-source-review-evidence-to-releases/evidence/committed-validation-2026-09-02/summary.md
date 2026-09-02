# Committed-source validation summary

## Accepted source

- Initial source-review implementation: `59db30ab31a00a94e9cab9c303f1d77276567ca4`
- Root Cargo evidence refresh: `4e50537f38f45dc6a34b7dde9fbae2e99e7aa1ab`
- Explicit optional default: `b5065660757828a8d31b6840fcf4eb78a5cc8de4`
- Accepted source and Nix closure repair: `60eaee08c9e5692560133fa401de0d9fcf31b358`

## Accepted checks

- `crunch-release-core`: 261 tests pass, including 26 focused source-review tests.
- Source-review shell: 9 tests pass.
- Release command: 20 tests pass.
- Release CLI: 166 tests pass and one fixture writer remains ignored.
- Machine contracts: PASS, with 24 contracted and 57 classified surfaces.
- Strict first-party Clippy: PASS with `-D warnings`.
- Tiger Style Nix gate: PASS without allowances or scope reduction.
- Formatting and `git diff --check`: PASS.
- Cairn validation: valid with no findings.
- Tracey: 155/155 references covered.
- Proposal, design, and tasks gates: PASS; all 18 tasks are complete.
- Durable-publication adoption: PASS after exact Cargo and flake binding refreshes.
- `nix flake check --no-build -L`: PASS.

## Full-check boundaries

The ordinary `nix flake check -L` reaches the unchanged remote Rust source import blocker:

```text
error: hash mismatch importing path '/nix/store/3r2nwafkx9xha0y6xsd0w0557ba0c294-rust-src-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu';
         specified: sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=
         got:       sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=
```

The local Nextest derivation now includes the tracked promoted proof evidence and all checked fixture roots. It starts 6,032 tests and passes the first 305. It then stops at the unrelated `evaluator_budget_cli::cancellation_is_terminal_and_reaps_a_late_worker` test because the child command unexpectedly returns success. This change does not skip or weaken that test.

These two blockers are outside the source-review policy, identity, signature, bundle, and release-verification paths. They do not contradict the accepted focused checks. Exact commands, full logs, and status files are in this directory.

## Decision

Accept the source-review release binding and archive the change. Preserve both full-check blockers as explicit non-green evidence.
