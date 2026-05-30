## Phase 1: Implementation

- [x] [serial] Add CLI parsing and shell for `mantle self-build --cargo-free --out <dir>`. r[rust_package_planning.cargo_free_self_build_command]
  - Evidence: `src/main.rs` adds the flag set and dispatches to `src/cargo_free_self_build.rs`; `README.md` documents the operator path.
- [x] [serial] Emit bounded audit evidence: copied binary, full receipt, source digest, Cargo guard status, smoke output, and summary. r[rust_package_planning.cargo_free_self_build_command]
  - Evidence: `src/cargo_free_self_build.rs` writes `mantle`, `receipt.json`, `stderr.txt`, `status.txt`, `smoke-*.txt`, `non-claims.txt`, and `meta.json`; pueue task 85 produced a real bundle with 599 units, 0 failed, Cargo marker absent, and binary BLAKE3 `4d9e93b5583180d00905a08a5e228f652386400c35687c3b0bcef2e6b47bfbd6`.
- [x] [serial] Add positive and negative CLI tests for tiny fixture build and Cargo guard failure. r[rust_package_planning.cargo_free_self_build_command]
  - Evidence: `tests/cargo_free_self_build_cli.rs` includes a positive tiny `mantle` fixture and a negative fake-`rustc` path that invokes ambient Cargo and must fail closed.
- [x] [serial] Validate change gates and focused Rust tests. r[rust_package_planning.cargo_free_self_build_command]
  - Evidence: `cairn/archive/2026-05-30-cargo-free-self-build-command/evidence/verification.md` records baseline, focused test, format, whitespace, and real self-build command results.
