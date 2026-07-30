## Why

Mantle can validate a StageX lineage manifest but deliberately returns `STAGEX_LINEAGE_PROVIDER_NOT_MATERIALIZED`. The checked receipt is `scaffold-only` and contains digest-shaped placeholders rather than observed seed, graph, provider, and audit identities. Consequently `seed-full.stagex-lineage` remains a StageX blocker even though the source-built native provider is selected for the narrower Guix-style boundary.

The missing work is an executable lineage: start from the audited hex0 seed and declared environmental assumptions, complete the protected transition through self-hosted TinyCC, native musl, and binutils, then publish that bounded intermediate provider with a real receipt.

## What Changes

- Implement StageX lineage materialization instead of returning the current fail-closed placeholder error.
- Bind the audited hex0 seed, transition tools, stage graph, source records, output identities, and environmental assumptions with BLAKE3.
- Move into protected execution before claimed provider construction and prohibit unrecorded `/bin/sh`, BusyBox, bwrap, checkout discovery, host compiler/linker, or source fallback.
- Require explicit absolute manifest and transition-root inputs plus an absent absolute output path.
- Publish target-prefixed tools, headers, libraries, and provider metadata with no-replace atomic rename.
- Replace the scaffold receipt only after a complete real lineage build and independent receipt validation.
- Keep final native GCC admission, compiler correctness, and Mantle self-build outside this intermediate claim.

## Dependencies

- `close-early-native-bootstrap-parity`.
- The protected StageX transition through authenticated binutils. Final-native parity remains separate evidence and is not a prerequisite for this intermediate provider.

## Impact

- **Files**: `bootstrap/stagex-transition-lineage.{ncl,json}`, `src/{main,bootstrap_source_root,bootstrap_parity,stagex_provider,stagex_transition,stagex_binutils}.rs`, the checked provider receipt, evidence, tests, and decision records.
- **Testing**: manifest/core validation; positive and negative protected-exec tests; real hex0-to-provider run; fallback mutation tests; parity CLI; Cairn gates.