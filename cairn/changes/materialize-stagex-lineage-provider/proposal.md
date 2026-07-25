## Why

Mantle can validate a StageX lineage manifest but deliberately returns `STAGEX_LINEAGE_PROVIDER_NOT_MATERIALIZED`. The checked receipt is `scaffold-only` and contains digest-shaped placeholders rather than observed seed, graph, provider, and audit identities. Consequently `seed-full.stagex-lineage` remains a StageX blocker even though the source-built native provider is selected for the narrower Guix-style boundary.

The missing work is an executable lineage: start from the audited hex0 seed and declared environmental assumptions, construct every transition stage through the completed native toolchain, protect the handoff from undeclared executables and fallback paths, and publish a real normalized provider plus receipt.

## What Changes

- Implement StageX lineage materialization instead of returning the current fail-closed placeholder error.
- Bind the audited hex0 seed, transition tools, stage graph, source records, output identities, and environmental assumptions with BLAKE3.
- Move into protected execution before claimed provider construction and prohibit unrecorded `/bin/sh`, BusyBox, bwrap, checkout discovery, host compiler/linker, or source fallback.
- Replace the scaffold receipt only after a complete real lineage build and independent receipt validation.

## Dependencies

- `close-early-native-bootstrap-parity`.
- `close-final-native-toolchain-parity`.

## Impact

- **Files**: `bootstrap/{stage0-posix,stage0-inventory,seed-full}.ncl`, `src/{bootstrap_source_root,protected_exec_seccomp,self_build}.rs`, CLI wiring, lineage core types, evidence/checkers, tests, and trust-root documentation.
- **Testing**: manifest/core validation; positive and negative protected-exec tests; real hex0-to-provider run; fallback mutation tests; parity CLI; Cairn gates.