# Verification gates for the pool change (2026-11-02)

Scope: S13's gate runs on the current tree. S13 is not claimed complete.

## Passed

```text
cargo test -p crunch-remote-core -p crunch-remote-app -- --test-threads 1
  -> ok: 10, 119, 1, 0, 1 tests across the crate targets
cargo test -p mantle --test remote_transfer_production -- --test-threads 1
  -> ok. 34 passed
cargo test -p crunch-build --lib -- --test-threads 1
  -> ok. 789 passed
cargo test -p mantle --test example_projects signed_cache_roundtrip_substitutes_through_an_admitted_trusted_cache
  -> ok. 1 passed
strict clippy (crunch-build, crunch-pipeline, mantle --tests, -D warnings) -> exit 0
cargo fmt --check (crunch-build, crunch-pipeline, mantle) -> exit 0
cairn validate -> issues [], change_issues [], receipt recorded
```

## No-std boundary (passed with an alternate nightly)

The devshell's active rustc (1.96.0-nightly-2026-03-21) has no
`wasm32-unknown-unknown` std and no rustup. The store also contains
`rust-default-1.96.0-nightly-2026-04-08`, which includes the wasm target; the
check ran with that toolchain first on `PATH`:

```text
bash scripts/check-no-std-core.sh  -> exit 0
  [11/12] API shape check OK
  [12/12] ownership check OK
```

The ownership step required classifying fourteen touched std sources; the
entries were added to `openspec/specs/functional-core/evidence/ownership-review.md`
(pool core and wasm/store/example sources `unrelated`, remote pool shell and
protected-build runner `adapter-only`).

## Nickel and operator checks

```text
cargo test -p mantle --test stdlib_tests            -> ok. 29 passed
scripts/check-operator-command-contract.sh          -> PASS (commands=214)
```

## Ordinary gate

`bash scripts/check-first-party-quality.sh` passed fmt and Clippy, then failed
in the serialized workspace tests on the known host flake
`workspace_shell::tests::production_workspace_local_dispatch_stack_reaches_owned_watch_cancel`
("watch sandbox did not start"; 788 passed, 1 failed). The same test also
failed on the committed tree earlier in this session and passed in other runs,
so the gate is not green on this host.

## Non-claims

- S13 is not complete: the ordinary gate is not green because of the host
  watch-sandbox flake, although its fmt and Clippy steps passed and the focused
  suites, no-std boundary, Nickel contracts, and operator contract are green.
- S14 stays ordered behind S13 and is unchecked.
