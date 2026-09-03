# Archive verification

- Cairn archived the active change with receipt BLAKE3 `db0136903f6ee981d691cbdbce307772162a27e26462b1e5210c32582146b66b`.
- Cairn reported archive mutation-manifest BLAKE3 `390280f1de39150adf09cbed819ab2d397a7d8d4095c7daaec7ab7aa75270f1b`.
- The operator corrected Cairn's known `1970-01-01` date to `2026-09-02` before validation.
- The active change path is absent.
- The archived task file has 11 checked tasks and no unchecked task.
- The canonical remote-build spec contains all four synchronized requirement IDs.
- Post-archive Cairn validation reports `"valid": true`.
- Post-archive Tracey reports 155/155 referenced.
- The accepted focused gates remain green. The full Nix build remains bounded by the recorded remote Rust-source hash mismatch.

`archive-files.blake3` lists each regular archive file except itself and `archive-manifest.blake3`. `archive-manifest.blake3` is the BLAKE3 of that sorted file manifest.
