## Why

Some host executables remain unavoidable at proof or sandbox setup boundaries. When that happens, Mantle should not treat them as invisible assumptions. Each host tool must be declared with its role, path, BLAKE3 digest, and version evidence, and protected execution should reject undeclared or mismatched tools before they can affect a strict proof.

## What Changes

- Define a host-tool attestation inventory for strict proof and sandbox setup paths.
- Verify absolute executable paths, BLAKE3 digests, role names, and bounded version output before use.
- Thread the accepted inventory digest into proof and build reports.
- Extend protected-exec denial diagnostics for undeclared, relative, unreadable, or digest-mismatched executable paths.

## Impact

- **Files**: stage0 inventory validation, protected exec policy, proof manifest/report fields, docs, and Cairn build-correctness spec delta.
- **Testing**: positive declared host-tool fixture; negative undeclared exec, digest mismatch, missing role, and relative path fixtures; Cairn validation and gates.

## Out of Scope

- Proving the correctness of declared host tools.
- Treating host-tool inventory acceptance as full-source bootstrap evidence.
