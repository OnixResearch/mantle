# Remaining blocker after local implementation

Date: 2026-06-29

## Question

Can `stabilize-self-hosting-witness-replay` be marked complete, synced, and archived after the local path-normalization, GCC hardening, and witness diagnostics changes?

## Inspected evidence

- `focused-validation-after-gcc-2026-06-29.md` records:
  - `nix develop -c cargo fmt --check -p mantle`
  - `nix develop -c cargo test -p mantle --bin mantle self_build::tests::` with 155 passing tests
  - `nix develop -c cargo test -p mantle --bin mantle witness_rebuild::tests::` with 24 passing tests
  - `git diff --check`
  - `cairn validate` plus proposal/design/tasks gates passing.
- `gcc-bootstrap-repro-attempt-20260629-025616.md` records two fresh GCC bootstrap validation runs from separate store/state pairs passing and converging on store entry `zq7sdyjfbv9v7njjgb62n1b4bnn0prda-gcc`.
- Earlier GCC evidence (`gcc-bootstrap-repro-attempt-20260629-020045.md`, `...021212.md`, and `...023754.md`) records the fixed frontiers: stage binutils dynamic-loader failure and nondeterministic generated manpage output.
- The implementation now adds stable self-build bootstrap aliases, a self-build rustc remap wrapper, GCC deterministic environment/metadata normalization, stage-binutils loader wrappers, deterministic generated-manpage placeholders, exact witness mismatch diagnostics, and a bounded bootstrap-divergence diagnostic that reports convergence or a `gcc.drv` root when logs/proof metadata expose it.

## Decision

No. The change remains active and must not be synced or archived yet. Local unit/Cairn checks are current and the two-run GCC bootstrap convergence proof now exists, but the checked tasks still require proof that has not been produced in this session: a full self-build fixed-point proof with binary leak scan evidence, fresh release evidence, and a separate-machine Aspen witness replay.

## Owner

Mantle self-build/release-evidence implementation.

## Next action

Run the full self-build fixed-point proof with binary leak scan evidence, then package fresh release evidence and rerun the Aspen witness replay. Only mark tasks complete after those transcripts exist in this change's evidence directory.
