# Aspen1 Self-hosting Witness Replay Blocker

Date: 2026-06-29

## Question

Can Aspen1 act as a separate-machine witness for `provider-bound-release-evidence-2026-06-28-provider-remap-fixed`?

## Inspected evidence

Generated artifacts were inspected under local `target/aspen1-witness-final/` and `target/aspen1-witness-driver-exact/` after copying summaries and audit metadata back from Aspen1.

Key observations:

- Provider fixed-point replay on Aspen succeeded and produced matching provider stage digest `aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20`.
- Aspen self-hosting proof reached an internal fixed point: `stage1_equals_stage2: true`.
- Aspen self-hosting stage digest was `bbaf4413111a5e0d9b9a751ee662ace330313cd1a4e96fc6d9bcd12cf353ca47`.
- The published release request expected `binaries/02-stage2-mantle` digest `70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703`.
- The witness command failed closed before writing sidecars with a digest mismatch diagnostic.
- Forcing the same rustup nightly and exact static busybox did not change the Aspen self-hosting digest.
- The final binary comparison showed generated Cargo path drift in `.rodata`:
  - local: `/tmp/cargo-target/x86_64-unknown-linux-musl/release/build/nickel-lang-parser-30857036f500e0c6/out/grammar.rs`
  - Aspen: `/tmp/cargo-target/x86_64-unknown-linux-musl/release/build/nickel-lang-parser-9482673c61447cbe/out/grammar.rs`
- GCC bootstrap output paths differed in proof logs:
  - local: `j18i4pgrw10q5f99j9djqcm705dgzsyl-gcc`
  - Aspen: `7rqgji1ircyclbz7n9gxw9j344brhp7r-gcc`

## Decision

Aspen1 cannot honestly be imported as a witness for the current release bundle because one published output does not match byte-for-byte. The provider proof is useful separate-machine evidence, but the full release witness remains blocked by self-hosting bootstrap/Cargo path nondeterminism.

## Owner

Mantle self-build/release-evidence implementation.

## Next action

Implement `stabilize-self-hosting-witness-replay`: normalize self-build bootstrap/Cargo/generated path identity, make GCC bootstrap output deterministic, keep exact-byte witness acceptance, then package a fresh release and rerun Aspen witness replay.
