# Hardware simulation implementation checkpoint

Date: 2026-07-14

This checkpoint proves only the completed profile/core/shell tasks recorded in `tasks.md`. It does not prove end-to-end compile, link, smoke, sharing, or selective invalidation.

## Completed task evidence

### I1 — boundary inventory

The implementation keeps typed hardware semantics in `crunch-hardware-simulation-core`, bounded OS/tool operations in `crunch-hardware-simulation`, and scheduling/store behavior in the existing generic `crunch-build` dynamic-plan worker. The design and `docs/hardware-simulation.md` record the frontend-neutral boundary and the Nix-produced seed assumption.

### I2 — typed profile and pure validation

`tests/fixtures/hardware-simulation/profile/{contracts.ncl,default.ncl}` define a closed typed Nickel profile. `invalid-unknown-field.ncl` and `invalid-zero-timeout.ncl` are negative fixtures. The pure `no_std` core validates typed sources, tops, source sets, exact cohort members, bounds, smoke cases, outputs, support tier, and non-claims.

### I3 / V1 — pinned source fixtures and closure checks

The profile pins test-owned local Git fixtures and recursive/sentinel BLAKE3 identities. Core tests prove selected closure inclusion, unrelated-fixture exclusion, and revision/digest/undeclared-edge rejection. Shell tests prove deterministic Git fixture materialization and reject invalid destinations/source kinds.

Pinned revisions:

- selected IP: `414a2f114880151657f02ca84624ce05efb4f9c1`
- selected VIP: `48659ef30e7948725b4373fe20f8585839e9634f`
- unrelated VIP: `831f20bb94d19bf8cce69ee79c6d457742234793`

### I4 — exact tool cohort

The final typed cohort identity is `mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c`. The declared closure digest is `32e93e2530eb0b3afcf3fd7062c1543441e02b5ba027a161e76631dcf31f31ed`. It uses Nix/store Verilator 5.046, Clang/LLD 21.1.8, Verilator runtime, and static BusyBox. `nix-produced-open-source-tool-cohort` is an explicit seed boundary; no source-bootstrap claim is made.

### Partial V2 evidence — bounded generated-plan validation

V2 remains unchecked because its I5 dependency is incomplete. Core tests nevertheless prove a positive independent-compile/one-link/two-smoke plan and negative undeclared-output, missing-file, unsupported-command, duplicate-unit, and bound-overflow cases. A capability-gated probe produced 12 generic units and was accepted by `crunch_build::dynamic_plan::decode_validated_plan_v1` and the native worker:

```text
profile_ref=mantle-hardware-profile://blake3/29ae88cd1abea071db01d724167876f4caef643344cce36e1d0b6e634ffb5c0f
cohort_ref=mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c
plan_blake3=dbecc70f0b3c58d97884867a86cf357dffad71452cb02212766490e38ce747c9
generic_plan_blake3=899f838c432bd51b7ddf55be5edbc574c06e229d668ea73dc4d4284d2632ddc3
units=12 roots=2 actions=13
```

The native build report recorded canonical plan digest `899f838c432bd51b7ddf55be5edbc574c06e229d668ea73dc4d4284d2632ddc3` and all 12 unit IDs as accepted.

## Focused validation

Command:

```sh
nix --option secret-key-files '' develop -c cargo test \
  -p crunch-hardware-simulation-core \
  -p crunch-hardware-simulation --lib
```

Result:

```text
crunch-hardware-simulation: 7 passed; 0 failed
crunch-hardware-simulation-core: 9 passed; 0 failed
```

Command:

```sh
nix --option secret-key-files '' develop -c cargo clippy \
  -p crunch-hardware-simulation-core \
  -p crunch-hardware-simulation \
  --all-targets --no-deps -- -D warnings
```

Result: exit 0.

## Capability-gated probe and exact blocker

Strict bubblewrap execution of the final static-shell cohort ran Verilator successfully:

```text
exit=0
files=16
bytes=190187
generated_blake3=dc3f36f05ec76104bc500f00764b06345df810bd8c76bb99dc776da11fbac552
```

The generic worker accepted the resulting plan, mounted the declared source/tool closure, and reached compile dispatch. Compile then failed because source placeholders in dynamic-unit environment values are not resolved by the generic worker:

```text
clang++: error: no such file or directory: '{{mantle-source:generated.sources}}/Vtiny_adder.cpp'
clang++: error: no input files
```

This is an exact bounded substrate blocker, not evidence of successful compile or smoke execution. I5–I10 and V3–V6 remain unchecked. No shared-result, selective-invalidation, elapsed-time, performance, or release-readiness claim is made.

## Additional blocker

The pinned development toolchain does not have `wasm32-unknown-unknown`; a target check fails with `error[E0463]: can't find crate for 'core'`. The core is authored as `no_std`, but this checkpoint does not claim wasm-target validation.

## Cairn checkpoint validation

The sibling Cairn flake could not be evaluated because its mutable path input changed during Nix evaluation. The already-built local Cairn binary was therefore used for the repo-local structural and advisory gates.

### `cairn validate --root .`

```text
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 30,
  "valid": true
}
```

### `cairn gate proposal prove-hardware-simulation-build-flow --root .`

```text
{"stage":"proposal","valid":true,"verdict":"PASS","issues":[],"receipt_hash":"07aa9db667f9cbdc409744626d13ff784620e467464ac11c1db33d994241ca79"}
```

### `cairn gate design prove-hardware-simulation-build-flow --root .`

```text
{"stage":"design","valid":true,"verdict":"PASS","issues":[],"receipt_hash":"355fde40fab08d418ce4217d7090efb164e7a9cba7a5b33dc4e265217bcf0b56"}
```

### `cairn gate tasks prove-hardware-simulation-build-flow --root .`

```text
{"stage":"tasks","valid":true,"verdict":"PASS","issues":[],"receipt_hash":"00a8c73b6df58c1cfe8c169be7494b5133b83b7c67bd61c99e0a11d1b349d7bf"}
```
