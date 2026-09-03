# Build-planning core

Mantle plans build routes and concurrency from explicit observations.

## Boundary

`crunch-build-planning-core` is a `no_std + alloc` functional core. It owns:

- route eligibility and preference;
- ordered rejected-route reasons;
- typed planning blockers;
- requested, observed, policy, fallback, and executor job limits;
- BLAKE3-bound effect plans;
- plan freshness and exact-effect checks.

The shell owns all observations and effects. It can inspect stores, source state, archives, remote candidates, doctor reports, platforms, keys, network state, executors, and host parallelism. It then supplies bounded facts to the core.

The core cannot open a remote session, redeem a credential, query a substituter, mutate a store, execute a build, or publish an output.

## Route order

The compatibility order remains:

1. local cache;
2. trusted substitution;
3. archive import;
4. source bundle;
5. remote builder;
6. local build;
7. preflight error.

Each rejected route retains a typed blocker and the accepted reason code. Several failures remain separate and ordered.

## Parallelism

The deterministic job policy receives:

- an optional requested job count;
- an observed host-parallelism value, unavailable state, or conversion failure;
- a configured policy cap;
- an optional executor cap;
- explicit zero and unavailable policies.

`crunch-pipeline` calls host available-parallelism only when the user did not supply a job count. It converts the observation before it calls the core.

## Effect execution

A successful plan contains a fact recheck followed by one route-specific effect. The effect identifies the selected route and the exact fact preimage.

Before execution, the shell compares current facts with the plan. Drift stops execution. The shell also rejects an effect not present in the plan, a wrong effect identity, or a changed route.

The shell does not silently re-plan during execution.

## Remote projection

The remote adapter derives candidate identity from a Mantle-owned `crunch-remote-core::RemoteCommand`. It adds explicit credential, capability, source, output-trust, network, and upload facts.

This projection does not open a session or redeem a credential.

## Checks

```sh
nix develop -c cargo test -p crunch-build-planning-core
nix develop -c cargo test -p crunch-pipeline
nix develop -c cargo -Zscript scripts/check-build-planning-architecture.rs --self-test
nix develop -c cargo -Zscript scripts/check-build-planning-architecture.rs --root .
nix build "path:$PWD#checks.x86_64-linux.build-planning-architecture" --no-link -L --builders ''
nix build "path:$PWD#checks.x86_64-linux.build-planning-core-wasm" --no-link -L --builders ''
```

## Non-claims

A route decision does not prove that its effect ran. An accepted observation does not prove build correctness or output trust beyond supplied facts.

This boundary does not prove route optimality, remote-worker correctness, reproducibility, or release eligibility.
