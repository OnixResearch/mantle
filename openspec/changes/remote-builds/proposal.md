## Why

All crunch builds are local bwrap sandboxes. For personal use across multiple machines, you need to offload builds to a more powerful machine, or build on a different architecture. Nix solves this with `--builders` (SSH-based remote builds). crunch needs an equivalent.

The build infrastructure is ready: `Builder`, `Worker`, `BubblewrapBuildService` all operate on derivations and produce `PathInfo`. The gap is a `BuildService` implementation that delegates to a remote crunch instance instead of a local sandbox.

## What Changes

- **Remote build service**: A new `BuildService` implementation that sends a derivation to a remote crunch instance over SSH (or a custom protocol), waits for the build, and retrieves the output NAR + PathInfo.
- **Builder configuration**: `--builders` flag or config file entry specifying remote machines: `ssh://user@host?system=x86_64-linux&max-jobs=4`.
- **Dispatch routing**: The `DispatchBuildService` gains a third arm: local bwrap, fetcher, or remote. Routing is based on system match and load.
- **Store synchronization**: After a remote build completes, the output NAR is pulled into the local store. This reuses the existing `crunch store pull` / NAR ingestion path.
- **Authentication**: SSH key-based auth for remote builders. The remote machine runs `crunch` and has its own store.

## Capabilities

### New Capabilities
- `remote-build`: Offload derivation builds to remote machines over SSH
- `builder-config`: Declare available remote builders with system type and job limits
- `multi-arch-build`: Build for a different system by routing to a matching remote builder

### Modified Capabilities
- `build`: Gains `--builders` flag for remote dispatch
- `store`: Remote build outputs are pulled into local store automatically

## Impact

- **Files**: New `crates/crunch-build/src/remote.rs` (remote BuildService), `src/main.rs` (--builders flag), config file schema
- **APIs**: New `RemoteBuildService` implementing existing `BuildService` trait
- **Dependencies**: Possibly `russh` or similar for SSH transport, or shell out to `ssh`
- **Testing**: Integration test with two local crunch stores simulating remote build. Unit tests for dispatch routing logic.
