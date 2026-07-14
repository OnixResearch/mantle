# Hardware simulation reference slice

Mantle keeps HDL semantics outside the scheduler and store. The checked reference slice lives in:

- `tests/fixtures/hardware-simulation/profile/default.ncl`: typed profile, source pins, exact Nix/store tool cohort, bounds, support tier, and non-claims.
- `crates/crunch-hardware-simulation-core`: pure `no_std` profile, source-closure, action-graph, smoke-result, and evidence logic.
- `crates/crunch-hardware-simulation`: bounded filesystem/Git/tool observation and strict bubblewrap shell adapter.

The profile is `heavy-capability-gated`. Its current cohort is Verilator 5.046, Clang/LLD 21.1.8, and a static BusyBox shell on `x86_64-linux`. These paths are Nix-produced seed inputs; Mantle does not claim source-bootstrap provenance for them.

## Fast validation rail

```sh
nix --option secret-key-files '' develop -c \
  cargo test -p crunch-hardware-simulation-core \
             -p crunch-hardware-simulation --lib

nix --option secret-key-files '' develop -c \
  cargo clippy -p crunch-hardware-simulation-core \
               -p crunch-hardware-simulation \
               --all-targets --no-deps -- -D warnings
```

The tests cover positive and negative typed-profile decoding, pinned local-Git materialization, demand-driven source closure, drift/edge rejection, bounded plan shape, smoke-result consistency, action invalidation, and count/byte-based evidence semantics.

## Heavy rail status

Current evidence proves strict-sandbox Verilator generation, deterministic normalized generated sources, generic `mantle-plan-v1` validation, and native-worker plan admission. End-to-end compile/link/smoke remains blocked: native dynamic-plan source inputs are registered as derivation inputs, but source paths embedded in unit environment values are not resolved by the worker. The observed compile argument remained the literal `{{mantle-source:generated.sources}}/...` path. Compile, smoke, shared-hit, and selective-invalidation claims therefore remain unproven and their Cairn tasks stay unchecked.

## Non-claims

This slice does not claim commercial simulator or license-server support, FPGA/ASIC correctness, physical design or timing closure, production remote-farm throughput, DVCon benchmark reproduction, or any elapsed-time speedup.
