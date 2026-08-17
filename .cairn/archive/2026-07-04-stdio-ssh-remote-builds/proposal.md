## Why

A full P2P transport is not required for useful distributed builds. A hardened stdio protocol, carried directly or over SSH, can support real remote machines while preserving the same frame semantics, output-trust checks, and input-sync rules.

## What Changes

- Promote stdio and SSH-stdio remote-build bindings from fixture seams to supported operator transports.
- Keep stdout reserved for length-prefixed protocol frames; send logs and diagnostics through bounded stderr/log frames.
- Complete cheap handshake, authorization, capability reporting, and request validation before expensive store scans or input walks.
- Add integration coverage for local child and SSH-stdio command execution with protocol-corruption and timeout negatives.

## Impact

- **Files**: remote serve/client CLI, stdio frame reader/writer, SSH command assembly, log bounds, timeout handling, docs, and Cairn remote-builds spec delta.
- **Testing**: positive stdio/SSH-stdio exchange; negative stdout pollution, oversized input, invalid sequence, child failure, timeout, and capability mismatch.

## Out of Scope

- Peer discovery.
- NAT traversal.
- Coordinator-based worker queues.
