# Change: Restore offline self-build source closure

## Why

The checkout-local `vendor-deps/` payload had drifted behind `Cargo.lock`: offline metadata stopped at missing `cap-fs-ext 4.0.2`. Regenerating the complete locked Cargo directory source repaired that boundary, but the canonical fixed-point proof then exposed two independent source-closure defects. The proof toolchain could not compile an unstable `char::MAX_LEN_UTF8` use, and the fixed self-build staging allowlist omitted tracked `config/` bytes consumed by `crunch-store` through `include_str!`.

A passing online checkout build does not detect either omission because it can use the ambient Cargo cache and the unstaged working tree. Self-build needs one explicit, checksum-verified Cargo directory source plus every build-relevant tracked root before it can make a current fixed-point claim.

## What Changes

- Regenerate the ignored checkout-local `vendor-deps/` directory from the locked Cargo graph and prove resolution with a fresh empty `CARGO_HOME`, offline mode, and `.cargo/vendor-config.toml`. r[bootstrap_inventory.self_build_source_closure]
- Keep self-build compatible with the pinned proof nightly by replacing the unstable UTF-8 width constant with its stable semantic equivalent. r[bootstrap_inventory.self_build_source_closure]
- Keep Linux atomic no-replace publication available on the bootstrap musl target by routing all four call sites through one syscall-backed shell helper with positive and negative race tests. r[bootstrap_inventory.self_build_source_closure]
- Add tracked `config/` policy payloads to the fixed staged-source allowlist and extend staging tests to prove required policy bytes are copied while non-allowlisted roots remain excluded. r[bootstrap_inventory.self_build_source_closure]
- Rerun the canonical fixed-point proof with strict later-stage hermeticity and the documented materialized-input fallback required by this host. r[bootstrap_inventory.self_build_source_closure]
- Preserve exact baseline, blocker, proof, lifecycle, and narrow non-claim evidence before changing any support statement. r[bootstrap_inventory.self_build_source_closure]

## Impact

- **Public CLI:** no new command or option.
- **Self-build semantics:** staged source now includes the tracked generated policy bytes required by the current workspace.
- **Local prerequisite:** `vendor-deps/` remains an ignored, explicit source input; this change repairs and validates the current checkout-local payload but does not make a fresh clone self-contained.
- **Compatibility:** the source compiles on the repository-pinned proof nightly and bootstrap musl target without enabling another unstable feature or weakening atomic no-replace publication.
- **Non-claims:** source closure and a successful fixed-point proof do not prove compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.
