# Native package contract parity

## Status

Source `7126fc98b86c6b49ff1247aeaf39bf92e0fb89ac` is published on `fix/package-contract-drift-20260905`.
The full native package check passed for that exact source. The installed output is valid.
Lifecycle completion remains pending. Package acceptance does not grant downstream admission.

Prerequisite: `5dfb37becbd09b895e84b8ba4af911a7f7513d91`.
The historical complete failure inventory remains in `evidence/package-test-fixtures-2026-09-05.md`.
ADR 0120 records the decisions and rejected alternatives.

## Diagnosis and repair

| Target | Contract-based correction |
| --- | --- |
| `examples_inventory` | Catalog and README now include both missing probes, with prerequisites and bounded non-claims. |
| `foreign_import_cli` | The unrelated fixture has a valid output/name binding. Malformed and missing-input assertions use their existing adapter categories. A separate negative checks invalid output semantics. |
| `integration_build` | The assertion compares the actual store basename. Shell builtins replace unavailable `mkdir`/`cat`. Runtime errors no longer become skips. The test requires successful execution, exact bytes, and no source backfill. |
| `machine_schema_contracts` | Exact unique cohort membership replaces the obsolete count. Missing and duplicate members reject. |
| `offline_build_runbook_docs` | Whitespace normalization preserves current non-claims without demanding obsolete future-work text. Missing non-claims and overclaims reject. |
| `operator_diagnostics` | The existing portable profile owner and parser supply reviewed projections. Eleven missing commands and existing release-review flags are documented. Stale generated output rejects without rewrite. |
| `remote_transfer_production` | The worker decoder preserves line framing and rejects duplicate fields or oversized reports. Fixtures inherit owned descriptors instead of replacing a fixed slot. Capture success, failed-build truth, and output rejection remain required. |
| `removed_system_cli` | Complete bounded coverage replaces the insufficient file limit. One exact reviewed fixture-source metadata field is data, not a dependency. Other coupling still rejects. Raw module inventory still causes no actions or store creation. |
| `store_gc_cli` | Mantle enumerates declared inspection layers, then uses ordinary composed lookup. Generic Snix cache listings remain near-only. |

The enumeration budget counts shadowed rows before deduplication. Reopened base reads preserve trust, precedence, generation, and no-backfill checks.
The build fixture drops its builder before reopening Redb. No base write capability or host shell alias was added.

## Gateway authority repair

The unchanged global guard found six violations in `src/remote_gateway.rs`, both here and at the published prerequisite.
The gateway now receives a store-owned `GatewayStore`, not a broad handle or raw services.
It exposes object reads, bounded NAR ingestion, and identity-bound imported-object persistence.
It cannot provide build, GC, source admission, signing, root registration, or raw services.

`GatewayNar` is constructed only after an actual ingest. A private actor-local token binds it to that open store instance.
An explicit negative first showed that a different store could reuse the observation. The repaired boundary rejects that transfer.
Node, digest, and size substitution also reject before metadata persistence.
The application keeps signature verification and checks current request authority again before final persistence.

The positive integration uses a signed symlink NAR and exact reopened PathInfo/node identity.
Imports preserve existing castore-only, non-root semantics. The initial test incorrectly expected a physical export; the corrected test requires its absence.
These controls do not prove every NAR node shape, arbitrary Nix compatibility, live revocation, crash recovery, or production isolation.
Rejected post-ingest requests can leave unindexed received objects. No accepted path or physical output is invented.

## Guard source preparation

The NAR guard had a stale direct `write_nar` expectation after the Rust-cache capability split.
It now checks the caller's `render_nar` delegation and the existing store-owned Snix writer. Missing-edge controls reject.
The source-commit and package-checksum expectations remain unchanged.

An isolated Cargo manifest selects exact `nix-archive = 0.1.0`, seeded from the repository lock.
Offline Cargo vendoring produced its source and file checksums. Only that crate was copied into ignored `vendor-deps/nix-archive` for the guard.
The observed source commit is `14362ab589daa4869bda744d4fbe26a1914b5491`.
The package checksum is `70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1`.
This sparse input is not a complete self-build vendor tree, a new product dependency, or a source-authenticity grant.

## Focused evidence

Final focused command: task 10096, completed successfully, 2026-09-05 22:59:58–23:11:50 -0400.
It used the pinned package development environment and an isolated Cargo target.

| Scope | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| examples inventory | 17 | 0 | 0 |
| foreign import | 18 | 0 | 1 |
| integration build | 12 | 0 | 0 |
| machine schema | 5 | 0 | 0 |
| runbook | 4 | 0 | 0 |
| operator diagnostics | 19 | 0 | 0 |
| remote transfer | 14 | 0 | 0 |
| removed-system | 10 | 0 | 0 |
| store GC | 6 | 0 | 0 |
| gateway unit and adapter controls | 17 | 0 | 0 |
| gateway compile-fail capability controls | 6 | 0 | 0 |
| store overlay controls | 24 | 0 | 0 |

Filtered and ignored tests are not execution evidence.
Strict scoped Clippy passed for Mantle and `crunch-store`.
The unchanged store-capability guard reported `external runtime findings=0`; its positive and negative self-tests passed.
The NAR self-test and source guard passed. The guard reported 8,867 scanned files in this checkout.
Generator checks reported `operator command contract: PASS (commands=188)`.
Pinned Tiger Style, lifecycle validation, design gate, and whitespace checks passed before source commit.
The earlier worker decoder control and both example self-tests passed in their separate retained logs.

## Native acceptance attempt

Task 10107 passed on 2026-09-05, 23:14:32–23:53:43 -0400. Elapsed time: 39 minutes 11 seconds.
The final Nix exit status was zero. This was not a timeout.
Library tests passed: 188 passed, zero failed.
Both CLI targets passed: 2,568 passed, zero failed, 72 ignored per target.
All nine repaired targets passed with the same counts in the focused table.
The evaluator target passed 15 tests, including the production release fault-hook control.
Nested subprocess summaries are not additional target totals.

Exact command:

```text
timeout --signal=TERM --kill-after=30s 60m nix build 'git+file:///home/brittonr/git/OnixResearch/mantle?ref=fix/package-contract-drift-20260905&rev=7126fc98b86c6b49ff1247aeaf39bf92e0fb89ac#default' --no-link --print-out-paths --builders '' --cores 2 -j 1 -L
```

The package retains `cargo test --release --locked --no-fail-fast`.
No test suppression, trust-key change, signature bypass, or production fault hook is added.
The build phase took 14 minutes 19 seconds. The check phase took 24 minutes 9 seconds.
The package completed installation and fixup.
`nix path-info` confirmed `/nix/store/ggqkhg64ca3fy1pnk1m4xbhcw266swic-mantle-0.1.0` as valid.
Its NAR hash is `sha256-lG3VzGNOU1AEukuNWg4mFGbeTmZzTvaq7g6v16Fejmg=`.
The installed binary passed `__operator-contract --mode check` for 188 commands.
Later evidence and lifecycle commits do not imply another native build of their tips.

## Raw-log identities

| File | BLAKE3 |
| --- | --- |
| `native-package.log` | `2f966e5e75f69b4f3f4d96d76bc289d4be111a96ee0b77529a05137e34b4185e` |
| `acceptance-final.log` | `e23e6e284f6003b197c866b1cb2bb7b17cab828f35b67b415f415a9f8143a7d2` |
| `tigerstyle-final.log` | `52d78e447b5bfa79101fdd6b3d1eb268c864df3aa87f2f6f63647c378d9419ba` |
| `native-output.json` | `3ed99fa5e13c35351de21c2c8a629e380e9cd8d667ed8bdfee1eeba870ae5998` |
| `operator-and-gateway-baseline.log` | `3a4f4e71e20d916ab8e4dde0c1a55290aa6eb57e3db4e8d8010f96f80140d3c0` |
| `cross-store-negative-baseline.log` | `d490ca610f307e4141d50a801059d2d4f190abd93c51591b2c7f552922861251` |
| `capability-prerequisite-baseline.log` | `e63baaa226957fa1c0e8c225d5f408facc22ee00c3cd4116a2e15efd459cd75f` |

These identities bind log bytes. They do not authenticate actors or grant promotion authority.

## Evidence location and non-claims

Durable raw logs are in the operator-owned Neural Stream primary checkout at `.pi/drains/2026-09-05-mantle-contract-drift/`.
They are not retained only in this worktree. Historical failed attempts remain separate from final acceptance logs.
The earlier cold Nickel preparation exceeded its inner command timeout scope; it is not claimed as a whole-operation ten-minute bounded run.

Native package success does not prove complete workspace/transitive coverage, reproducibility, compiler correctness, or release eligibility.
Neither repository changes downstream pins, remote Nix trust, training, tensor materialization, or production promotion because of these observations.
