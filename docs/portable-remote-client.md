# Portable remote client

Mantle separates the client platform from the build target platform. A Darwin
client can evaluate Nickel and send concrete derivation facts to an admitted
Linux remote builder. The client does not send raw Nickel source as a remote
execution request.

## Supported boundary

The reviewed platform matrix is in `config/operator-surfaces.ncl`. The pure
admission and request planner is in `crates/mantle-portable-client-core/`.

On Darwin, `mantle build` supports these modes:

- `--plan`, which does not start a local executor.
- `--builder ENDPOINT --ticket-fd FD`, which selects the existing admitted
  remote-build path.

A remote build also requires trusted builder key material. Mantle validates the
remote capability, trust set, frontend-neutral payload kinds, target platform,
and logical store prefix before dispatch.

The remote response uses the existing signed `PathInfo` admission path. Mantle
imports the result into client state and materializes the requested output in
the configured physical store directory. The logical store prefix remains an
explicit request fact and does not have to be `/nix/store`.

## Stable blockers

Mantle rejects unsupported Darwin operations before state-directory mutation.
Important blocker codes are:

- `portable-remote-route-required`: a build would otherwise use local execution.
- `portable-local-executor-unsupported`: the command requires the Linux local
  executor.
- `portable-worker-server-unsupported`: the operation is a worker or server.
- `portable-bootstrap-unsupported`: the operation is bootstrap work.
- `portable-proof-unsupported`: the operation is proof work.
- `portable-raw-frontend-payload-rejected`: a request contains raw
  frontend-specific payload data.

## Validation

Run these checks from the repository root:

```sh
nix develop -c cargo test -p mantle-portable-client-core
nix develop -c cargo check -p mantle-portable-client-core --target aarch64-apple-darwin
nix develop -c cargo check -p mantle-portable-client-core --target x86_64-apple-darwin
nix develop -c cargo -q -Zscript scripts/check-portable-client-boundary.rs .
./scripts/check-operator-command-contract.sh
```

Cross-target compilation proves source and dependency portability for the core.
It does not prove native Darwin execution. Native Darwin command tests remain a
separate requirement when a Darwin runner is available.
