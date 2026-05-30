# Verification Evidence

## Commands

- `cargo -q -Zscript scripts/prove-cargo-free-fixed-point.rs --check --mantle-bin /home/brittonr/.cargo-target/debug/mantle --root .`
  - Result: preflight succeeded.
  - Reported schema `mantle-cargo-free-fixed-point-proof-v1`, repo root, default `/tmp` bundle path, host Mantle path, and rustc path.
- `cargo -q -Zscript scripts/prove-cargo-free-fixed-point.rs --check --bundle-dir target/cargo-free-fixed-point-proof/check --mantle-bin /home/brittonr/.cargo-target/debug/mantle --root .`
  - Result: failed closed with exit 1 because the requested bundle was inside source root.

- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid=true`; changes 1; specs validated 2; no issues.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate proposal cargo-free-fixed-point-proof --root .`
  - Result: PASS; receipt hash `235c484c3c8b2b9480e6d4cd6e4793e8fe86f479a00a7e5ee81a16c4b4954afc`.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate design cargo-free-fixed-point-proof --root .`
  - Result: PASS; receipt hash `fa87f5c97e37174c9ea8f132ce07bb728ab747d4f25747e8271b509e4d0f994d`.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks cargo-free-fixed-point-proof --root .`
  - Result: PASS; receipt hash `1d7a9eb15ee4f0ee5886761a308861abbd747d0fd2a6e4931452e788f4ca49eb`.

## Mismatch Investigation

- `pueue_log 43 --full`
  - Bundle: `target/cargo-free-fixed-point-proof/run-20260530T035318Z`.
  - Result: stage1 and stage2 both succeeded with 599 units and no failed units, but binary digests differed.
  - Stage1 digest: `7df31c1eb61968e230a0f68424b74e5bfc84a28e8be8393a18d9d9977402f8b1`.
  - Stage2 digest: `c195d1d5e58d2499656094f289b691e981c1bc88dce0b564da3c0b6afe32c86d`.
  - Root cause: the bundle was under source root `target/`, so stage2 source digests included stage1 evidence; separate per-stage execution roots also embedded stage-specific generated/build-script paths in debug metadata.
  - Runner fix: reject inside-root bundles by defaulting to `/tmp`, and reuse a single cleared execution-output root for both stages while copying produced binaries into stage evidence directories.

## Archive Notes

- `cairn archive cargo-free-fixed-point-proof --execute --root .` initially wrote `cairn/archive/1970-01-01-cargo-free-fixed-point-proof`; renamed it to `cairn/archive/2026-05-30-cargo-free-fixed-point-proof`.
- The archive move did not sync the delta requirement into the canonical spec; appended the archived rust-package-planning delta to `cairn/specs/rust-package-planning/spec.md` manually, then reran `cairn validate --root .` with `valid=true`.

## Successful Fixed-point Proof

- `pueue_log 46 --full`
  - Bundle: `/tmp/mantle-cargo-free-fixed-point-proof/run-20260530T041338Z`.
  - Result: `status=success`, `fixed_point=true`.
  - Stage1: 599 units, 0 failed, binary `/tmp/mantle-cargo-free-fixed-point-proof/run-20260530T041338Z/stage1/mantle`.
  - Stage2: 599 units, 0 failed, binary `/tmp/mantle-cargo-free-fixed-point-proof/run-20260530T041338Z/stage2/mantle`.
  - Shared binary BLAKE3: `99c4a052c7f2234e16568a24c267ed5483474bc336315193b3a1d49809efc239`.
  - Cargo marker files were absent for both stages.
  - Stage1/stage2 Mantle bin unit IDs, source digests, rustc arg digests, environment digests, and output artifact digests matched.
