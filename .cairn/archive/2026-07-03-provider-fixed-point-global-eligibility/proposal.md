## Why

The 2026-07-02 provider-bound release now carries a provider fixed-point proof bundle, external Aspen witness agreement, and a blocked full-release global reproducibility report. The blocker is specifically that `binaries/01-mantle` was treated as a provider handoff surface even though the provider proof verifier can prove strict source-built toolchain closure, stage1/stage2 fixed-point bytes, cargo-guard absence, successful receipts, and matching release artifact digest.

Operators need Mantle to either admit that artifact from explicit verifier facts or preserve a concrete blocker when those facts are missing or invalid.

## What Changes

- Admit a provider fixed-point release artifact as strict/fresh global surface evidence only when the existing provider proof verifier reports a valid proof and its stage binary digest matches the release artifact digest.
- Keep the helper fail-closed: missing verifier facts, invalid provider proof bundles, missing policy/meta digests, or stage/release digest mismatches still emit blocker-producing evidence.
- Rebase provider proof metadata paths to bundle-local copies when a proof bundle has been copied away from the original absolute build directory.
- Regenerate the 2026-07-02 full release universe evidence/report and record the exact bounded eligibility evidence.

## Impact

- **Files**: global reproducibility release helper, provider fixed-point proof verifier path resolution, focused tests, docs/release evidence, Cairn verification-evidence delta.
- **Testing**: focused positive/negative helper tests, provider verifier copied-bundle regression, real 2026-07-02 full-release evidence/report regeneration, formatting, Cairn validation/gates.
