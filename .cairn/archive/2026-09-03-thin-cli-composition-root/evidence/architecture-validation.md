# Architecture validation

Date: 2026-09-03

The maintained architecture rail accepted the repository. It rejected each adversarial source fixture with the expected owner, authority class, and dependency path.

```text
negative fixture rejected: path=fixtures/cli-application-architecture/negative/cli.rs owner=negative-fixture authority=cli dependency-path=clap::
negative fixture rejected: path=fixtures/cli-application-architecture/negative/snix.rs owner=negative-fixture authority=snix dependency-path=snix_
negative fixture rejected: path=fixtures/cli-application-architecture/negative/filesystem.rs owner=negative-fixture authority=filesystem dependency-path=std::{fsasdisk}
negative fixture rejected: path=fixtures/cli-application-architecture/negative/process.rs owner=negative-fixture authority=process dependency-path=std::process
negative fixture rejected: path=fixtures/cli-application-architecture/negative/async-runtime.rs owner=negative-fixture authority=async-runtime dependency-path=tokio
negative fixture rejected: path=fixtures/cli-application-architecture/negative/environment.rs owner=negative-fixture authority=environment dependency-path=std::{envasambient}
negative fixture rejected: path=fixtures/cli-application-architecture/negative/clock.rs owner=negative-fixture authority=clock dependency-path=std::time
negative fixture rejected: path=fixtures/cli-application-architecture/negative/random.rs owner=negative-fixture authority=random dependency-path=rand::
negative fixture rejected: path=fixtures/cli-application-architecture/negative/network.rs owner=negative-fixture authority=network dependency-path=std::net
negative fixture rejected: path=fixtures/cli-application-architecture/negative/provider.rs owner=negative-fixture authority=provider dependency-path=provider_sdk
negative fixture rejected: path=fixtures/cli-application-architecture/negative/store-service.rs owner=negative-fixture authority=store-service dependency-path=crunch_store
negative fixture rejected: path=fixtures/cli-application-architecture/negative/rendering.rs owner=negative-fixture authority=rendering dependency-path=tracing::
negative fixture rejected: path=fixtures/cli-application-architecture/negative/root-policy.rs owner=negative-fixture authority=domain-policy dependency-path=fn plan_
CLI application architecture verified: findings=0 root-production-lines=365 ports=12 negative-fixtures=13 compile-fail-fixtures=2
owners: core=crates/mantle-application-core application=crates/mantle-application inbound=src/cli_architecture/inbound_adapter.rs operation=src/cli_architecture/operation_adapter.rs presentation=src/cli_architecture/presentation_adapter.rs
boundaries: core-purity application-owned-ports adapter-direction explicit-composition typed-error-ownership presentation-separation no-std-targets
```

The exact output is in `evidence/validation-2026-09-03/architecture.log`.

Two Rust compile-fail cases also passed:

```text
running 2 tests
test crates/mantle-application/src/lib.rs - (line 29) - compile fail ... ok
test crates/mantle-application/src/lib.rs - (line 6) - compile fail ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

One case rejects an incomplete family-port set. The other rejects a Snix type in an application port contract.

The Nix architecture check passed in pueue task `2823`. Host application tests and the `wasm32-unknown-unknown` build also passed in that task.

This rail proves the checked source topology. It does not prove external effect success, provider correctness, deployment, or release eligibility.
