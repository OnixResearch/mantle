# Hardware simulation reference slice

Mantle keeps HDL semantics outside the scheduler and store. The checked reference slice lives in:

- `tests/fixtures/hardware-simulation/profile/default.ncl`: typed profile, source pins, exact Nix/store tool cohort, bounds, support tier, and non-claims.
- `crates/crunch-hardware-simulation-core`: pure `no_std` profile, source-closure, action-graph, smoke-result, and evidence logic.
- `crates/crunch-hardware-simulation`: bounded filesystem/Git/tool observation and strict bubblewrap shell adapter.
- `examples/hardware_simulation_plan.rs`: a frontend-only JSON request to generic `mantle-plan-v1` conversion boundary.

The scheduler and store remain frontend-neutral. They validate and execute the same declared-source and declared-unit-output placeholders used by any dynamic plan; they contain no HDL, simulator, Verilator, compile, link, or smoke special cases.

## Support tier and prerequisites

The profile is `heavy-capability-gated` on `x86_64-linux`. The proven cohort is:

- Verilator 5.046 at `/nix/store/3761xqvdyh2f5962f8ivl5lshp97y6fm-verilator-5.046`.
- Clang wrapper 21.1.8 at `/nix/store/hh6y3s72d21whp6q98h4dh0valxiaw69-clang-wrapper-21.1.8`.
- LLD 21.1.8 at `/nix/store/2c5r0kpn5yb8qsvfw77j8h63pklij8rq-lld-21.1.8`.
- Static BusyBox at `/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0`.
- Linux user namespaces and bubblewrap.
- A writable physical store, state directory, and temporary directory with enough space for generated C++, objects, and reports.

The cohort ref is `mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c`; its 53-root closure digest is `32e93e2530eb0b3afcf3fd7062c1543441e02b5ba027a161e76631dcf31f31ed`. The exact sorted closure is checked in at `tests/fixtures/hardware-simulation/tool-closure-paths.txt` and its digest is a fast-test invariant. These paths are Nix-produced seed inputs. Mantle does not claim source-bootstrap provenance for them.

## Fast validation rail

```sh
nix --option secret-key-files '' develop -c \
  cargo test -p crunch-hardware-simulation-core \
             -p crunch-hardware-simulation --lib

nix --option secret-key-files '' develop -c \
  cargo test -p mantle --test examples_inventory

nix --option secret-key-files '' develop -c \
  cargo check -p mantle --example hardware_simulation_plan

nix --option secret-key-files '' develop -c \
  cargo clippy -p crunch-hardware-simulation-core \
               -p crunch-hardware-simulation \
               --all-targets --no-deps -- -D warnings
```

The tests cover positive and negative typed-profile decoding, pinned local-Git materialization, demand-driven source closure, drift/edge rejection, bounded plan shape, typed smoke-result consistency, action invalidation, and count/byte-based evidence semantics.

For a prepared `HardwarePlanRequest`, generate the generic plan with:

```sh
nix --option secret-key-files '' develop -c \
  cargo run -p mantle --example hardware_simulation_plan -- \
  request.json plan.json
```

The output is ordinary `mantle-plan-v1` JSON. A producer derivation declares that file as a `dynamic_plan_output`; `mantle build` then validates and executes the compile/link/smoke graph through the generic worker.

## Real-tool evidence

The 2026-07-14 capability run used fresh state under `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh` and proved:

- deterministic strict-sandbox Verilator generation: 16 normalized files, 190,187 bytes, BLAKE3 `dc3f36f05ec76104bc500f00764b06345df810bd8c76bb99dc776da11fbac552`;
- generic plan digest `b1e30af6fd30c080a993e3d4505cd602bc962c484764ad4e3aa5591f82c202df`, with nine independent compile units, one link unit, two smoke units, and three dynamic roots;
- all compile units, the explicit simulator link root, and both parameterized smoke roots completed in the native strict sandbox;
- `mantle-hardware-smoke-result-v1` outputs for `0,0 -> 0` and `1,2 -> 3`, each with exit code zero, empty bounded stderr, typed profile/cohort/source/action/simulator refs, and explicit non-claims;
- selected-source revision `48c284fac87a9350de026902e134c16b6a66711c` changed the selected source, generated-source, profile, plan, compile, link, and smoke identities and rebuilt the graph;
- an unrelated-source change left the selected plan digest unchanged and caused zero worker executions: all producer, compile, link, and smoke outputs were accepted from the local cache;
- wrong-reference revision `d2cc93ee610863e22c73d4a1579f43d36501f006` built and linked but both smoke roots failed with exit code 4 and `reference mismatch`; no successful smoke output or smoke action-result publication was emitted.

The bounded report, receipt, action-result, attestation, log, and smoke-result paths are enumerated in `cairn/changes/prove-hardware-simulation-build-flow/evidence/final-lifecycle.md`.

## Remaining blocker

A clean-client, full-shared hardware hit is not claimed. The current input-addressed hardware graph is admitted by the ordinary local PathInfo cache before action-result discovery can select a shared result. A manually copied result index without PathInfo, receipt, policy, and reference-scan evidence failed closed, as designed. Generic HTTP action-result tests prove zero-executor clean-client reuse, rejection, conflict, and publication semantics, but that is not promoted to hardware-specific full-shared evidence. Tasks requiring that exact hardware proof remain unchecked.

## Non-claims

This slice does not claim commercial simulator or license-server support, FPGA/ASIC correctness, physical design or timing closure, production remote-farm throughput, DVCon benchmark reproduction, elapsed-time speedup, source-bootstrap provenance for the tool cohort, release readiness, or wasm validation.
