# Phase 2 evidence

- Compile: `cargo check -p mantle --lib --tests` exit 0 (CARGO_INCREMENTAL=0, dedicated target).
- Unit tests (`--bin mantle`): foreign_realization 13 passed, foreign_executable_plan 7 passed, foreign_realization_shell 2 passed; existing cache-only tests included.
- CLI: `--test cache_inputs_build_route` 6 passed (positive build, cache-offered root, missing input, untrusted signer, recompute refusal, source-bundle refusal). `--test foreign_import_cli` 18 passed, 1 pre-existing ignored.
- Store: `cargo test -p crunch-store --lib` 463 passed, 1 failed only because the long TMPDIR exceeded SUN_LEN; the same test passes with a short TMPDIR (`/datapool/mantle-store/t`).
- fmt and clippy: see clippy-fmt-validate.txt.
- no-std: not run to completion; the earlier attempt died in a build script before any classification check due to `/tmp` quota. Known baseline is the pre-existing step-12 failure.
- cairn validate and workflow status: clippy-fmt-validate.txt / cairn-gates.txt.
