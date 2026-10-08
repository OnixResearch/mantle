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

The remote response uses the existing signed `PathInfo` admission path.
Mantle verifies the requested output, content, builder key and signature,
logical store prefix, and artifact evidence in the host adapter before
admitting the output to client state and the configured physical store.
Physical bytes can remain after a failed partial admission; they are not an
admitted output. The logical store prefix is an explicit request fact and
does not have to be `/nix/store`.

The production client's active session and its child share the assigned job,
attempt, and fence within their processes. Reconnect or reassignment is
explicit, not an automatic effect-loop retry. The transfer manifest establishes
chunk scope; a checkpoint or acknowledgement is not verified receiver content
or signed output authority. See the
[transient-handle boundary table](remote-transfer.md#transient-handle-admission).

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
