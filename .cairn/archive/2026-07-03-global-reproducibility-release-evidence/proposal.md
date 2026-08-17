## Why

The 2026-07-02 provider-bound release now has scoped release evidence, Aspen external witness agreement, and a stage2-only global reproducibility report. Operators need a repeatable way to derive global reproducibility surface evidence from a release bundle and verification result instead of hand-writing JSON. The derivation must keep unsupported release artifacts blocked rather than promoting the whole release when strict/fresh evidence is missing.

## What Changes

- Add a release CLI helper that reads a digest-bound universe, policy, release evidence bundle, verification directory, and final release-verify JSON, then writes `mantle-global-reproducibility-surface-evidence-v1` entries.
- Derive eligible evidence for strict stage2 self-hosting release surfaces when the proof bundle records strict stage2 evidence and a policy-counted witness matches the artifact digest.
- Derive blocked/unsupported evidence for provider fixed-point handoff artifacts that are release-bounded but not strict global reproducibility surfaces.
- Record a full 2026-07-02 release-universe report showing `binaries/01-mantle` blocks while `binaries/02-stage2-mantle` remains accepted.

## Impact

- **Files**: release CLI parsing/dispatch, global reproducibility helper code, tests, docs/release note evidence, Cairn verification-evidence spec delta.
- **Testing**: focused release/global reproducibility CLI tests, positive stage2 evidence derivation, negative full-release blocker derivation, Cairn validation/gates.
