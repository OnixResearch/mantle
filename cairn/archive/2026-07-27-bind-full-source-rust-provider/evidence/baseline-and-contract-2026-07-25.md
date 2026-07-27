# Full-source Rust provider baseline and contract evidence

Date: 2026-07-25

## Scope

This evidence completes only tasks I1 and I2. It does not claim that the real
full-source Rust provider has materialized, that the shell is receipt-bound, or
that full-bootstrap parity is complete.

## Current mechanism registry

`bootstrap/rust-source-route-registry.ncl` is the typed mechanism-level source
of truth for the five observed routes:

| Mechanism | Full-bootstrap candidate | Disqualifying observation |
|---|---:|---|
| `musl-host-full-source` | yes, conditionally | no completion without a validated `mantle-full-source-rust-provider-binding-v1` receipt |
| `source-built-gnu-host` | no | host role does not bind the admitted musl native provider |
| `imported-provider` | no | import validation does not prove current construction |
| `ambient-host` | no | compiler/linker selected through ambient discovery |
| `fetched-rust-compatibility` | no | prebuilt Rust compatibility route |

The smallest discriminating probe is the current construction receipt: it must
validate against an independently observed
`mantle-full-source-provider-admission-v2` identity and must bind the exact
native artifact matrix, Rust source IDs, stage receipt identities, final Rust
artifact identities, host/target roles, offline source policy, zero ambient
discovery, zero fallback events, and zero seed exceptions.

Focused registry evidence was produced by pueue task 554:

```text
running 2 tests
test rust_source_route_registry_selects_only_the_full_source_musl_candidate ... ok
test rust_source_route_registry_disqualifies_compatibility_and_import_routes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out
```

## Real musl-host baseline

The baseline invoked the committed `bootstrap/rust-source-musl-host-plan.ncl`
with the admitted native provider at:

```text
/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/q366f9q6n1silgmv8w0c2ypxxw4p4yw3-full-source-seed-toolchain
```

The run acquired and authenticated `rust-1.90.0` source, built real mrustc and
minicargo with the source-root musl C/C++ provider, compiled the Rust standard
library through 89.5%, and reached the LLVM host-tools boundary. It failed
closed before provider publication. Pueue task 479 recorded the exact first
blocker:

```text
/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/q366f9q6n1silgmv8w0c2ypxxw4p4yw3-full-source-seed-toolchain/bin/ld:
attempted static link of dynamic object
`/nix/store/29mmnqpc1p3iv8wj0lpvicajy3jsbx87-zstd-1.5.7/lib/libzstd.so`
collect2: error: ld returned 1 exit status
make[4]: *** [utils/TableGen/CMakeFiles/llvm-min-tblgen.dir/build.make:124:
bin/llvm-min-tblgen] Error 1
```

This is an ambient Nix dynamic-library leak and therefore disqualifying evidence,
not a completed provider. The source-route generator now sets both
`LLVM_ENABLE_ZSTD=OFF` and `CMAKE_DISABLE_FIND_PACKAGE_zstd=ON`. The preserved
scratch rerun is diagnostic only; a fresh committed-source materialization is
still required by V2.

## Pure binding contract

`src/full_source_rust_binding.rs` adds two pure contracts without weakening the
existing compatibility provider schema:

- `mantle-full-source-rust-provider-binding-v1` binds independently observed
  native admission, exact native tool/runtime paths and BLAKE3 identities,
  Rust stage sources, observed build receipts, final Rust artifacts,
  host/target roles, and zero-exception policy.
- `mantle-full-source-toolchain-closure-binding-v1` binds the native closure
  policy digest, Rust provider policy digest, native admission/output digests,
  and construction-receipt digest while rejecting every seed exception.

Focused positive and negative evidence was produced by pueue task 545:

```text
running 6 tests
full_source_binding_accepts_complete_native_and_rust_evidence ... ok
full_source_binding_rejects_host_target_role_substitution ... ok
full_source_binding_rejects_missing_stage_receipt ... ok
full_source_binding_rejects_native_artifact_role_substitution ... ok
full_source_binding_rejects_wrong_native_provider_identity ... ok
full_source_closure_binding_rejects_seed_exception ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1596 filtered out
```

The contract is functional core only. It performs no filesystem, process,
network, clock, or environment access. The pending shell task must independently
observe and hash inputs, call this contract, and publish create-new sidecars.

## Bounded non-claims

- The real provider is not yet materialized.
- The diagnostic scratch rerun is not accepted V2 evidence.
- Live source acquisition in the baseline is not authenticated offline source
  state for promotion.
- The admitted native provider report does not by itself prove the Rust route.
- Imported or fetched providers cannot satisfy this change.
- Compiler correctness and whole-bootstrap correctness are not claimed.
