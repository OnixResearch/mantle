# Post-review examples inventory validation

Purpose: make the previously summarized `examples_inventory: 7 passed` claim reviewable inside the current follow-up diff.

Compilation evidence: `cairn/archive/2026-06-01-examples-supported-contract/evidence/implementation-validation-2026-06-01.md` records `cargo test -p mantle --test examples_inventory -- --nocapture` with `test result: ok. 7 passed; 0 failed; 0 ignored`.

Current execution uses the already-built libtest binary to avoid unrelated bin-target relinking noise from this harness environment.

Binary: `/home/brittonr/.cargo-target/debug/deps/examples_inventory-f099cf4a3bfa12d9`

## examples_inventory libtest binary --nocapture

```text

running 7 tests
test examples_catalog_rejects_stale_crunch_branding_in_user_docs ... ok
test examples_catalog_rejects_readme_omissions_and_stale_paths ... ok
test examples_catalog_rejects_duplicate_ids_and_paths ... ok
test examples_catalog_rejects_invalid_tier_and_silent_skip ... ok
test documented_crunch_ncl_identifier_is_allowed ... ok
test examples_catalog_supports_lane_inventory_for_docs ... ok
test examples_catalog_covers_checked_in_user_facing_examples ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


```

Exit: `0`
