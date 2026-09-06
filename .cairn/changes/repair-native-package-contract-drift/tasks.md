# Repair tasks

Evidence: `evidence/native-package-contract-parity-2026-09-05.md` binds source `7126fc98b86c6b49ff1247aeaf39bf92e0fb89ac`, exact commands, target counts, raw-log identities, and non-claims.
The native package passed in 39 minutes 11 seconds. All nine repaired targets passed without test suppression.

## Diagnosis and contracts

- [x] [serial] Record focused baselines and classify all nine failures against their accepted owners. The evidence diagnosis table retains each correction and its boundary. r[native_package_parity.verification]
- [x] [serial] Correct foreign-import fixtures without changing admission policy. Valid, malformed, missing-input, and output-binding controls pass: 18 passed, zero failed, one ignored. r[native_package_parity.fixtures]

## Inventories and guards

- [x] [serial] Restore complete example catalog coverage and honest classifications. Inventory target: 17 passed, zero failed. Both probe self-tests passed separately. r[native_package_parity.inventories]
- [x] [serial] Replace stale machine cardinality with exact unique membership checks. Positive and negative controls: five passed, zero failed. r[native_package_parity.inventories]
- [x] [serial] Regenerate and review operator inventory and documentation through declared producers. Freshness passed for 188 commands. All 19 operator controls, including stale-artifact rejection, passed. r[native_package_parity.inventories]
- [x] [serial] Repair bounded module coverage and runbook assertions without weakening non-claims or admitting raw inventory. Removed-system target: ten passed. Runbook target: four passed. r[native_package_parity.fixtures] r[native_package_parity.inventories]

## Store and receipt boundaries

- [x] [serial] Restore base-only build and inspection through narrow capabilities. Reopened reads, exact output, and no-backfill controls passed. Build target: 12 passed. Store GC target: six passed. r[native_package_parity.overlay]
- [x] [serial] Run unsafe-base, signature, shadow, prefix, generation, and authority controls. All 24 overlay controls passed. The unchanged global capability guard reported zero findings. r[native_package_parity.overlay]
- [x] [serial] Preserve worker capture through coordinator reporting. Capture, rejection, cleanup, failed-build truth, and descriptor controls passed. Remote transfer target: 14 passed. The decoder control passed separately. r[native_package_parity.capture]
- [x] [serial] Replace gateway broad authority with bounded import/query capabilities. All 17 gateway and six compile-fail controls passed. Cross-store observations, changed metadata, invalid signatures, and denied authority reject. Both global store and NAR guards passed with positive/negative controls. r[native_package_parity.gateway]

## Acceptance

- [x] [serial] Run all nine focused targets, changed-boundary controls, generator freshness, strict scoped Clippy, Rust formatting, pinned Tiger Style, and git diff --check. All passed. No Nix source changed. r[native_package_parity.verification]
- [x] [serial] Commit source and run the exact default Nix package with cargo test --release --locked --no-fail-fast. Nix exited zero. Both CLI targets passed 2,568 tests with 72 ignored each. The installed output and its command-contract check passed. r[native_package_parity.verification]
- [ ] [serial] Publish evidence, validate lifecycle gates, review sync/archive dry runs, and integrate only after all required gates pass. r[native_package_parity.verification]
