# Design: bind-rust-build-script-link-metadata

Mantle will keep the existing host-artifact topology rail and extend `bind_all_build_script_metadata`.

1. Parse-time validation keeps link metadata bounded:
   - `rustc-link-lib` supports bare names plus `static=`, `dylib=`, and `framework=` forms.
   - `rustc-link-search` supports bare paths plus `dependency=`, `crate=`, `native=`, `framework=`, and `all=` forms.
   - Empty, whitespace-containing, path-traversal-like, or modifier/rename-heavy forms block as malformed for this slice.
2. Target binding is deterministic:
   - Sort/dedup is already applied during metadata parsing.
   - For each consumed custom-build metadata summary, append `-L <value>` for each search entry and `-l <value>` for each lib entry after cfg/env binding.
   - Recompute `rustc_args_digest_blake3` after mutation.
3. Receipts stay on the existing JSON rail:
   - build-script metadata runs continue to expose captured link metadata.
   - target unit execution receipts expose changed argument digest and success/failure status.
4. Fail-closed behavior remains pre-target-rustc for malformed metadata because parsing happens immediately after build-script execution.
