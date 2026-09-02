# Tasks: Adopt the Nickel 1.17 evaluator cohort

## Dependency and source cohort

- [x] [serial] Pin `nickel-lang 2.2.0` and `nickel-lang-core 0.18.0`. r[mantle.nickel_toolchain.cohort]
- [x] [serial] Add an exact Nickel CLI `1.17.0` source input at commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`. r[mantle.nickel_toolchain.cohort]
- [x] [serial] Regenerate Cargo and Nix lockfiles only through Cargo and Nix commands. r[mantle.nickel_toolchain.cohort]
- [x] [parallel] Add a cohort guard that rejects old, mixed, or floating Nickel dependencies. r[mantle.nickel_toolchain.cohort]

## Vendor and adapter update

- [x] [serial] Refresh vendored Nickel sources through the repository-owned importer. r[mantle.nickel_toolchain.vendor]
- [x] [parallel] Verify the vendor manifest, checksums, licenses, source commit, and Rust requirement. r[mantle.nickel_toolchain.vendor]
- [x] [depends:mantle.nickel_toolchain.vendor] [serial] Adapt evaluator APIs for parsing, diagnostics, imports, and direct deserialization. r[mantle.nickel_toolchain.boundary]
- [x] [parallel] Guard build, store, scheduler, and evidence cores against upstream runtime types. r[mantle.nickel_toolchain.boundary]

## Compatibility and evidence

- [x] [parallel] Run valid derivation, import, contract, and direct-deserialization fixtures. r[mantle.nickel_toolchain.compatibility]
- [x] [parallel] Add malformed, missing-import, failed-contract, oversized, budget, and redaction negative fixtures. r[mantle.nickel_toolchain.compatibility]
- [x] [serial] Update bootstrap and release evidence with the exact evaluator and vendor identities. r[mantle.nickel_toolchain.evidence]
- [x] [parallel] Add stale-manifest, mixed-cohort, and weakened-non-claim evidence failures. r[mantle.nickel_toolchain.evidence]

## Validation

- [x] [serial] Run focused evaluator, vendor, bootstrap-source, and evidence checks. r[mantle.nickel_toolchain.validation]
- [x] [serial] Run formatting, Clippy, relevant workspace tests, Cairn gates, and relevant Nix checks. r[mantle.nickel_toolchain.validation]
