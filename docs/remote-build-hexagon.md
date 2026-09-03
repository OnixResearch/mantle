# Remote-build hexagon

Mantle separates deterministic remote-build meaning from host effects.

## Boundary

`crunch-remote-core` is a `no_std + alloc` functional core. It owns:

- bounded remote commands and admitted output facts;
- protocol and build-service admission;
- realization and remote-build identities;
- fencing, retry, transfer-limit, resource-limit, and output-trust decisions;
- deterministic phases, events, effects, observations, outcomes, and receipt preimages.

The core does not read files, start processes, use a network, read environment variables, observe a clock, create random identifiers, verify credentials, access a store, use an async runtime, or render output.

`crunch-remote` is the application shell. It executes one effect through one capability port. It then gives the typed observation to the core before it requests the next effect.

## Capability ports

The application shell defines separate ports for:

1. transport and bounded input transfer;
2. attempt persistence and lease changes;
3. executor launch;
4. store output admission;
5. credential verification;
6. clock observation;
7. random identifier generation;
8. telemetry publication.

`RemotePortSet` is the visible composition root. A port error becomes a failed observation. The core applies retry policy and does not fabricate success.

## Adapters

Outer adapters remain responsible for vendor and host types:

- `crunch-build::distributed::snix_adapter` owns Snix build requests, results, and I/O errors.
- `src/remote_hexagon/transport_adapter.rs` owns stdio and SSH transport wrappers.
- `src/remote_hexagon/executor_adapter.rs` owns local and external-batch executor wrappers.
- Store, telemetry, attempt, credential, clock, and identifier wrappers have separate modules.
- `src/remote_hexagon.rs` projects existing wire frames, PathInfo records, substitution reports, and admission reports into Mantle-owned facts.

Existing protocol bytes remain authoritative. Compatibility tests serialize existing frames and core projections, then compare the bytes.

## Active behavior owners

This extraction does not redefine active gateway, resource-policy, transfer, telemetry, external-batch, nominal-type, failure-debug, or Trellis requirements. Their adapters supply accepted facts to the new boundary.

## Checks

Run:

```sh
nix develop -c cargo test -p crunch-remote-core
nix develop -c cargo test -p crunch-remote
nix develop -c cargo test -p crunch-build distributed
nix develop -c cargo test -p mantle --bin mantle remote_build::
nix develop -c cargo check -p crunch-remote-core --target wasm32-unknown-unknown
nix develop -c cargo -Zscript scripts/check-remote-hexagon.rs --self-test
nix develop -c cargo -Zscript scripts/check-remote-hexagon.rs --root .
nix build .#checks.x86_64-linux.remote-hexagon-architecture -L
nix build .#checks.x86_64-linux.remote-core -L
nix build .#checks.x86_64-linux.remote-core-wasm -L
```

The architecture check rejects Snix, store, Tokio, filesystem, path, process, environment, clock, random, network, credential, CLI, and rendering authority in the core. It also checks that Snix types stay in the named adapter.

## Non-claims

A core plan does not prove that an effect ran. An adapter observation does not prove worker honesty, transport confidentiality, compiler correctness, whole-output trust, reproducibility, or release eligibility.
