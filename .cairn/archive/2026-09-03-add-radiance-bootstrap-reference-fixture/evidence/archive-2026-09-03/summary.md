# Radiance reference archive evidence

Date: 2026-09-03

## Result

The Cairn change is complete and archived at
`.cairn/archive/2026-09-03-add-radiance-bootstrap-reference-fixture/`.
The eight accepted Radiance requirements are present in
`.cairn/specs/bootstrap-inventory/spec.md`.

The archive dry run and execution passed. Post-archive Cairn validation,
Tracey coverage, machine-contract validation, and the architecture checker
also passed. See `archive-dry-run.json`, `archive-execute.json`, and
`post-archive-validation.log`.

## Committed proof

The final V16 proof ran from implementation commit
`5e35c8a518e871cbbf844598b274ddb7842c9316`.

- Disposition: `match`.
- Fixed-point BLAKE3: `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`.
- Fixed-point size: 1,285,492 bytes.
- Receipt BLAKE3: `da80e6d4f2fbf4adb82aabfaeef37e4f61e8a628024d999672458f0589e571c9`.
- Audit BLAKE3: `2da1ee4274c085598aef78258cbaec4fc84cc764d80db469356c198160dd004e`.
- Protected executions: 50 allowed and 0 denied.
- Proof-time network requests: 0.
- Replay verification: pass.

The proof and artifacts are in
`../live-proof-v16-2026-09-03/`.
Committed-source validation is in
`../committed-validation-2026-09-03/`.

## Validation boundary

Focused Rust, WASM, Clippy, Tiger Style, Nickel, Nix, architecture,
machine-contract, Tracey, and Cairn checks passed. The repository-wide checks
that remain blocked are recorded in `../final-validation-2026-09-03/`.
They are pre-existing failures outside the optional Radiance reference.

This evidence proves only the bounded convergence described by the accepted
requirements. It does not prove compiler correctness, source trust, semantic
equivalence, universal reproducibility, or release eligibility. It does not
alter or extend the V98 promoted proof.
