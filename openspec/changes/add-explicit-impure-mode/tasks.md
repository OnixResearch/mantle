## Phase 1: CLI and mode plumbing

- [x] [serial] Add `impure` to the hermeticity mode model and preserve it through build pipeline/self-build orchestration.
- [x] [depends:mode] Add `--impure` to build-entry commands and reject `--strict-hermetic --impure` conflicts before execution.
- [x] [depends:mode] Emit impure mode and typed impure audit facts in human output, JSON reports, and attestations.

## Phase 2: Proof rejection and docs

- [x] [depends:mode] Make release verification, witness agreement, and deterministic proof classification reject impure material for existing proof classes.
- [x] [depends:mode] Document `--impure` as a development/compatibility escape hatch that is not proof-eligible.
- [x] [depends:mode] Add positive/negative tests for mode plumbing, CLI conflict, JSON labeling, and proof rejection.
- [x] [depends:verification] Validate and archive this OpenSpec after implementation lands.
