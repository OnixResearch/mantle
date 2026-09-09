## Fixture adoption

- [ ] [serial] Add `fault-injection` as a pinned dev-dependency and record transport `crates.io`, plane `validation` in the dependency catalog. r[mantle.io_fault.fixtures]
- [ ] [serial] Wrap store path reads, NAR serialization writes, build-commit fsync, and cache writes with `fallible!` in test builds. r[mantle.io_fault.fixtures]
- [ ] [serial] Add a trigger function that records the injection site for fixture assertions. r[mantle.io_fault.fixtures]

## Determinism

- [ ] [serial] Add a setup guard storing an explicit counter value and restoring `u64::MAX` after each fixture. r[mantle.io_fault.determinism]
- [ ] [serial] Keep `SLEEPINESS` at zero in every fixture and in CI configuration. r[mantle.io_fault.determinism]

## Boundary

- [ ] [serial] Document that the dependency is test-only and that injection coverage grants no sandbox-hermeticity, store-correctness, or release-eligibility claim. r[mantle.io_fault.boundary]

## Verification

- [ ] [parallel] Add a positive fixture proving an unchanged build and cache cycle with the default counter. r[mantle.io_fault.verification]
- [ ] [parallel] Add negative fixtures firing at each wrapped site and asserting the declared bounded build-fact classification. r[mantle.io_fault.verification]
- [ ] [parallel] Add boundary fixtures asserting the annotation and serialized counter ownership. r[mantle.io_fault.verification]
- [ ] [serial] Run package, workspace, Clippy, Cairn, and Nix checks; append the komora-io reference under `## References` in `README.md`; document non-claims. r[mantle.io_fault.verification]
