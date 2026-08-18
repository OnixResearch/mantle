# Native closure frontier

Task-ID: H1
Covers: rust_package_planning.source_built_toolchain_closure.target_aliases

## Decision

Target-prefixed receipt-bound aliases are proven, but full native toolchain closure is not complete.

## Evidence considered

- Fixed-point proof bundle: `/home/brittonr/git/mantle-native-toolchain-target-alias-fixed-point-2026-06-16`.
- Toolchain manifest: `/home/brittonr/git/mantle/target/receipt-bound-native-toolchain-target-aliases/toolchain-closure-host-and-target-aliases.json`.
- Bundle `meta.json` reports:
  - `source_built_toolchain_closure.status = validated-enforced`
  - `source_built_toolchain_closure.claim = false`
  - `source_built_toolchain_closure.seed_exception_count = 8`
  - `source_built_toolchain_closure.source_built_member_count = 2`
  - `non_claims` still includes `not-source-built-toolchain-closure`.

## Remaining work

- Replace seed-exception host C/linker helpers with source-built closure members.
- Replace seed-exception musl target GCC/binutils helpers with source-built closure members.
- Add real sysroot/crt/runtime-library closure members with content digests instead of the current Rust-provider sysroot placeholder digest.
- Bind pkg-config and any target native helper discovery explicitly when packages need them.
- Only after those are source-built and receipt-bound should Mantle remove `not-source-built-toolchain-closure` for the full native closure.
- Release reproducibility, Crunch/bootstrap self-hosting, full Cargo compatibility, tests, doctests, and examples remain separate non-claims.
