# Evidence: adapter-neutral-foreign-derivation-import

## Scope

Task-IDs: I1, I2, I3, I4, I5, I6, I7, I8, V1, V2, V3, V4, V5
Covers: r[foreign_derivation_import.adapter_neutral_ir], r[foreign_derivation_import.pure_translation_core], r[foreign_derivation_import.store_prefix_rewrite_policy], r[foreign_derivation_import.import_receipt], r[foreign_derivation_import.package_index_boundary], r[foreign_derivation_import.fetch_and_cache_policy], r[foreign_derivation_import.sandbox_capability_audit], r[foreign_derivation_import.integration_boundary]

## Implementation evidence

Implemented in `src/foreign_derivation_import.rs` and registered from `src/main.rs`.

- I1: Defines `foreign-derivation-graph-v1`, `foreign-package-index-v1`, `foreign-derivation-import-receipt-v1`, named limits, graph node/source-payload/index/receipt fields, and canonical sorting before digesting.
- I2: `translate_foreign_graph(...)` is a pure deterministic in-memory core. It receives graph/index/policy data and returns `TranslatedGraph` plus `ImportReceipt` or deterministic diagnostics.
- I3: `TranslationPolicy` includes source prefixes, target prefix, rewrite toggles, builtin mappings, embedded source-payload rewrite permission, output recomputation mode, trusted cache scopes, and allowed sandbox capabilities.
- I4: `guix_like_hello_fixture()` and `nix_like_hello_fixture()` produce concrete derivation graph facts into the same IR. The Nix fixture is derivation-JSON-like graph data and does not model flake/evaluator semantics.
- I5: `plan_mantle_foreign_import(...)` is the thin Mantle-facing adapter over accepted translated artifacts and a package index. It emits a `mantle-foreign-derivation-adapter-plan-v1` data plan and no process invocations.
- I6: `PackageIndex` / `PackageIndexEntry` and `lookup_package(...)` provide generic by-name lookup over name/system/alias/root facts; unsupported frontend metadata fails closed.
- I7: Source mirrors and cache hints are policy data; untrusted cache hints are rejected unless their trust scope is allowed by policy.
- I8: Sandbox compatibility needs are per-node capabilities. Undeclared capabilities fail closed; declared capabilities produce adapter audit events.

Purity grep for the new core:

```text
native_grep pattern: std::fs|fs::|std::env|std::process|println!|eprintln!|read_to_string|read_dir|std::time|SystemTime|Command::|tokio|async
path: src/foreign_derivation_import.rs
result: No matches found
```

## Verification evidence

### V1/V2/V3/V4 — focused import-core and adapter tests

Command (pueue task 60):

```text
cargo fmt -p mantle
cargo test -p mantle --bin mantle foreign_derivation_import::tests:: -- --test-threads=1 --nocapture
```

Output summary:

```text
running 5 tests
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1126 filtered out; finished in 0.00s
```

Positive coverage:

- Guix-like and Nix-like hello fixtures translate through the same adapter-neutral IR.
- Equivalent graph traversal order produces the same raw and translated graph digests.
- Mantle adapter planning consumes translated artifacts without `guix`, `nix`, flakes, Nix expression evaluation, package-module evaluation, or any process invocation.

Negative coverage:

- Unsupported builtins, undeclared foreign references, stale receipt digests, malformed package indexes, oversized fields, unsupported mandatory features, and non-lowered flake/overlay metadata fail closed.
- Untrusted cache hints and undeclared sandbox capabilities fail closed.
- Embedded source-payload store rewrites require explicit policy permission.

### V5 — formatting and Cairn gates

Formatting and whitespace checks (pueue task 61):

```text
cargo fmt -p mantle --check
git diff --check
```

Output: task completed successfully.

Cairn validation and gates (pueue task 62):

```text
COMMAND=validate
{"stage":"validate","valid":true,"verdict":null,"issue_count":0,"receipt_hash":null}
COMMAND=proposal
{"stage":"proposal","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"7e869c911c5edb1ab7f31d8b442d59aba1a9cbad846dafa923f2aef42e301117"}
COMMAND=design
{"stage":"design","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"9626ab5f37587d014de318026ff0623f4a21288089d00db3e68c729e788a8ec2"}
COMMAND=tasks
{"stage":"tasks","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"23f89a1606a484b0582b6f698759f1780f292db926ce94f0660eff8e938430e4"}
```

## Lifecycle evidence

Sync/archive/post-archive validation (pueue tasks 63 and 64):

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync adapter-neutral-foreign-derivation-import --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
CAIRN_ARCHIVE_DATE=2026-07-03 nix run path:/home/brittonr/git/cairn#cairn -- archive adapter-neutral-foreign-derivation-import --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
...
"receipt_hash": "5deb8c5b11e0f50e1969f6b8b67ab2a9fc4d8726a91671fc9911912291f8599a"
{"valid":true,"issue_count":0}

nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
{"valid":true,"issue_count":0}
```

Accepted-spec repair validation (pueue task 66):

```text
The initial sync created `cairn/specs/foreign-derivation-import/spec.md` as a skeleton because the change used a full spec body. The archived full requirement text was copied into the accepted spec, then validation was rerun.

nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
{"valid":true,"issue_count":0}
```
