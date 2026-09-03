# Archive verification

- Cairn archived the active change with receipt BLAKE3 `069f326de9f5696a4aa628c9c5b33fd9bb5e1a3e6cf56b98119b8ac610cfb2c0`.
- Cairn reported archive mutation-manifest BLAKE3 `8c28eac8d19461b3e4784db175558f4ac17c74b2a72cbad3b61bf6366bdb4026`.
- The operator corrected Cairn's known `1970-01-01` date to `2026-09-03` before validation.
- The active change path is absent.
- The archived task file has 11 checked tasks and no unchecked task.
- The accepted scheduling and routing specs contain all four synchronized requirement IDs.
- Post-archive Cairn validation reports `"valid": true`.
- Post-archive Tracey reports 155/155 referenced.
- The accepted focused gates remain green. The full Nix build remains bounded by the recorded remote Rust-source hash mismatch.

`archive-files.blake3` lists each regular archive file except itself and `archive-manifest.blake3`. `archive-manifest.blake3` is the BLAKE3 of that sorted file manifest.
