# Tasks: Harden replayable witness test rails

## Phase 1: Close scratch-boundary coverage gaps

- [x] I1 Add direct unit coverage in `src/witness_rebuild.rs` for helper-owned
      `tmp/` / `cargo-target/` entries, proving accepted-directory,
      symlink-rejection, and non-directory-rejection behavior.
      [covers=release.verification.tech.witness.rebuild.scratch]
- [x] I2 Add or extend replayable witness CLI coverage so a symlinked helper-
      owned scratch entry fails before the workflow driver starts, not just a
      symlinked scratch-root path.
      [covers=release.verification.tech.witness.rebuild.scratch]

## Phase 2: Make no-launch claims observable

- [x] I3 Tighten `tests/release_cli.rs` fake-driver plumbing so the launch
      signal is a consumed seam: the fake driver must read the configured
      signal path and write it immediately on process start.
      [covers=release.verification.tech.witness.rebuild.testing]
- [x] I4 Update preflight-failure witness tests to assert both the rejection
      diagnostic and the absence of the consumed launch signal, so no-launch
      claims cannot pass vacuously.
      [covers=release.verification.tech.witness.rebuild.testing]

## Validation

- [x] V1 Run `cargo test -p crunch --bin crunch
      validate_existing_scratch_root -- --nocapture` and capture the output
      proving helper-owned directory, symlink, and non-directory cases behave
      as specified.
      [covers=release.verification.tech.witness.rebuild.scratch]
      Evidence: 2026-04-24 targeted unit rail passed with `5 passed; 0 failed`,
      including helper-owned directory acceptance plus symlink/non-directory
      rejection coverage.
- [x] V2 Run `cargo test -p crunch --test release_cli witness_rebuild_ --
      --nocapture` and capture the output proving helper happy-path coverage
      remains green while negative tests use a real launch-observation seam.
      [covers=release.verification.tech.witness.rebuild.scratch,release.verification.tech.witness.rebuild.testing]
      Evidence: 2026-04-24 targeted CLI rail passed with `10 passed; 0 failed`,
      including the new symlinked helper-owned scratch-entry rejection and
      launch-signal assertions.
- [x] V3 Run `openspec validate harden-replayable-witness-test-rails` after the
      implementation and evidence land.
      [covers=release.verification.tech.witness.rebuild.scratch,release.verification.tech.witness.rebuild.testing]
      Evidence: 2026-04-24 `openspec validate harden-replayable-witness-test-rails`
      returned `Change 'harden-replayable-witness-test-rails' is valid`.
