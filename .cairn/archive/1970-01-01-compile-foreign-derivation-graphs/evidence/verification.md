# Final verification

Task-ID: V1, V2
Covers: r[foreign_derivation_import.exact_graph_compilation], r[foreign_derivation_import.executable_plan]
Date: 2026-08-01

## V1 focused Rust verification

Pueue task: `7909`

```text
$ nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 2001 filtered out; finished in 0.00s

$ nix develop -c cargo test -p crunch-glue
test result: ok. 85 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ nix develop -c cargo test -p mantle --test foreign_import_cli
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Result: PASS.

## V2 documentation and diff verification

Pueue task: `7910`

```text
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
foreign import trust-model doc check passed
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
foreign import trust-model checker self-test passed
$ git diff --check
```

`git diff --check` exited with status 0 and no diagnostics.

Result: PASS.

## Additional focused checks

Strict first-party Clippy passed in pueue task `7897`:

```text
nix develop -c cargo clippy -p mantle --bin mantle --test foreign_import_cli --no-deps -- -D warnings
nix develop -c cargo clippy -p crunch-glue --all-targets -- -D warnings
```

Vendored `snix-castore` emitted one existing `dead_code` warning.

The machine-schema checker has unrelated baseline inventory debt. Pueue task `7901` reproduced the same StageX and source-build findings on baseline commit `36833dcb`. The changed worktree reports no foreign import source finding after its inventory update.

## V3 lifecycle validation

Pueue task: `7912`

The repository policy still lacks `nominal_identity_policy`. Therefore, these
commands used the current sibling Cairn policy at
`/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`.

```text
$ cairn validate --root . --policy current
"valid": true

$ cairn gate proposal compile-foreign-derivation-graphs --root . --policy current
"valid": true
"verdict": "PASS"

$ cairn gate design compile-foreign-derivation-graphs --root . --policy current
"valid": true
"verdict": "PASS"

$ cairn gate tasks compile-foreign-derivation-graphs --root . --policy current
"valid": true
"verdict": "PASS"
"task_done": 14
"task_todo": 1

$ cairn tracey coverage --root . --policy current
"referenced": 255
"requirements": 682
"valid": false
"verdict": "fail"
error: tracey coverage failed
```

Tracey has a pre-existing repository-wide blocker. Pueue task `7913` reproduced
`referenced: 255`, `requirements: 682`, and `valid: false` on baseline commit
`36833dcb`. The changed worktree output contains no missing or dangling
`foreign_derivation_import` requirement.

The validation and all change gates passed. Tracey did not regress, but its
repository-wide baseline debt remains outside this change.

After V3 was checked, pueue task `7914` reran the tasks gate:

```text
"task_done": 15
"task_todo": 0
"valid": true
"verdict": "PASS"
```

Cairn sync completed in pueue task `7915`. Post-sync Tracey ran in pueue
task `7918`:

```text
"referenced": 259
"requirements": 686
"valid": false
"verdict": "fail"
```

The four new accepted requirements and their four implementation links account
for both count increases. None of the four IDs occurs in the post-sync
`missing` or `dangling` sets. The unchanged repository-wide baseline debt still
prevents a global PASS.
