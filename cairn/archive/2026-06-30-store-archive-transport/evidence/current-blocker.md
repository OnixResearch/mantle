# Resolved blocker — Store archive transport

Date: 2026-06-30

## Previous blocker

The change was previously blocked because it required a real Mantle-native streaming archive format, CLI surfaces, store-level import/export/list behavior, and positive/negative tests rather than a pure-only model.

## Resolution

The final slice implemented the transport instead of stubbing it:

- `crates/crunch-store/src/archive.rs` defines and validates `mantle-store-archive-v1` with metadata-before-payload frames, store-prefix binding, deterministic closure export planning, trusted-signature policy, BLAKE3 payload digests, and native-only compatibility wording.
- `src/main.rs` and `src/store_cmd.rs` expose `mantle store archive export|import|list` with human/JSON reports and file/stdin/stdout stream support.
- `tests/store_archive_cli.rs` covers human and JSON output, stdin/stdout streaming, import skip behavior, strict unsigned export behavior, and native compatibility wording.
- Archive unit tests cover recursive closure planning, skip/import/reject decisions, bad magic, header/end count mismatches, prefix mismatch, unsigned strict policy, untrusted signatures, tampered payloads, truncated payloads, unsupported CA metadata, conflicting local PathInfo, missing closure facts, and non-seekable bounded payload draining.

## Evidence

See `cairn/changes/store-archive-transport/evidence/completion-2026-06-30.md` for current commands and passing results.
