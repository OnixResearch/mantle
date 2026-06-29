## Why

Aspen1 can replay the current provider-bound release request far enough to prove the provider fixed-point stage, but the full witness rebuild fails before signing because the self-hosting `stage2-mantle` digest differs from the published release artifact. The failure is useful: it shows the current release evidence remains provider-bound/local for the self-hosting artifact and cannot yet support an Aspen witness claim.

The observed drift is not source drift. The staged source path is the same logical source tree, the provider proof reaches a fixed point, and self-hosting reaches `stage1_equals_stage2: true` on Aspen. The remaining mismatch comes from bootstrap/self-build identity leaking through nondeterministic GCC output paths and Cargo-generated `OUT_DIR` names such as `nickel-lang-parser-<hash>/out/grammar.rs`.

## What Changes

- Normalize self-build bootstrap tool paths before invoking Cargo so content-addressed GCC/Rust/binutils paths do not perturb Cargo fingerprints or embedded generated-source paths.
- Add a self-build rustc wrapper/remap layer for `CARGO_TARGET_DIR`, build-script `OUT_DIR`, staged source roots, and bootstrap tool aliases while preserving real filesystem paths for build-script I/O.
- Harden `bootstrap/gcc.ncl` for deterministic output by fixing time/locale/umask/archive behavior and proving equivalent GCC builds converge across fresh stores.
- Keep `mantle release witness-rebuild` fail-closed on exact digest mismatch, but improve audit diagnostics so bootstrap path drift is identified without weakening witness acceptance.

## Impact

- **Files**: `src/self_build.rs`, `bootstrap/gcc.ncl`, release witness audit code in `src/release_evidence.rs`/`src/attest_cmd.rs` as needed, tests under `tests/` and focused unit tests near the touched modules.
- **Testing**: positive and negative unit tests for path remapping, GCC reproducibility smoke, self-build fixed-point proof, Aspen/local witness replay when available, Cairn validation/gates.
