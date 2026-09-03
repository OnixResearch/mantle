# Archive verification

- Cairn archived the active change with receipt BLAKE3 `7b51208d7218e71859033b223903b9d56e81c3d8e7e3436af10a9839fb031109`.
- Cairn reported archive mutation-manifest BLAKE3 `7ac7c41911a1148d69753c51427e319830147ec57d95aa88728ad4ce8bed3079`.
- The operator corrected Cairn's known `1970-01-01` date to `2026-09-03` before validation.
- The active change path is absent.
- The archived task file has 11 checked tasks and no unchecked task.
- The canonical Rust package-planning spec contains all four synchronized requirement IDs.
- Post-archive Cairn validation reports `"valid": true`.
- Post-archive Tracey reports 155/155 referenced.
- The accepted focused gates remain green. The full Nix build remains bounded by the recorded remote Rust-source hash mismatch.

`archive-files.blake3` lists each regular archive file except itself and `archive-manifest.blake3`. `archive-manifest.blake3` is the BLAKE3 of that sorted file manifest.
