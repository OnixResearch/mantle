## Fixture adoption

- [x] [serial] Add `fault-injection` as a pinned dev-dependency and record transport `crates.io`, plane `validation` in the dependency catalog. r[mantle.io_fault.fixtures]
- [ ] [serial] Wrap store blob reads, NAR stream writes, post-build action-result record pre-link fsync, and cache writes with `fallible!` in test builds. r[mantle.io_fault.fixtures]
- [ ] [serial] Add a trigger function that records the injection site for fixture assertions. r[mantle.io_fault.fixtures]

## Determinism

- [ ] [serial] Add a setup guard storing an explicit counter value and restoring `u64::MAX` after each fixture. r[mantle.io_fault.determinism]
- [ ] [serial] Keep `SLEEPINESS` at zero in every fixture and in CI configuration. r[mantle.io_fault.determinism]

## Boundary

- [x] [serial] Document that the dependency is test-only and that injection coverage grants no sandbox-hermeticity, store-correctness, or release-eligibility claim. r[mantle.io_fault.boundary]

## Verification

- [ ] [parallel] Add a default-counter store/NAR/cache fixture and prove an unchanged actual build/cache cycle with the existing smoke scenario. r[mantle.io_fault.verification]
- [ ] [parallel] Add negative fixtures at each wrapped site asserting the existing store/export error or cache-publication diagnostic, while preserving CA-required fail-closed publication. r[mantle.io_fault.verification]
- [ ] [parallel] Add boundary fixtures asserting the crate/file/line annotation, default counter restoration, and fixture-serialized counter ownership. r[mantle.io_fault.verification]
- [ ] [serial] Run package, workspace, Clippy, Cairn, and Nix checks; append the komora-io reference under `## References` in `README.md`; document non-claims. r[mantle.io_fault.verification]
