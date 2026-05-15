## Phase 1: CLI and mode plumbing

- [ ] [serial] Add `impure` to the hermeticity mode model and preserve it through build pipeline/self-build orchestration.
- [ ] [depends:mode] Add `--impure` to build-entry commands and reject `--strict-hermetic --impure` conflicts before execution.
- [ ] [depends:mode] Emit impure mode and typed impure audit facts in human output, JSON reports, and attestations.

## Phase 2: Proof rejection and docs

- [ ] [depends:mode] Make release verification, witness agreement, and deterministic proof classification reject impure material for existing proof classes.
- [ ] [depends:mode] Document `--impure` as a development/compatibility escape hatch that is not proof-eligible.
- [ ] [depends:mode] Add positive/negative tests for mode plumbing, CLI conflict, JSON labeling, and proof rejection.
- [ ] [depends:verification] Validate and archive this OpenSpec after implementation lands.
