## Phase 1: Audit current docs against shipped surfaces

- [x] Compare `README.md` against `src/main.rs` and current help output for `crunch`, `crunch build`, `crunch shell`, `crunch attest`, and `crunch release`.
- [x] Compare `docs/benchmark-suite.md` against the checked-in benchmark entry points and `openspec/specs/performance/spec.md`.
- [x] Compare `docs/bootstrap-stage0-inventory.md` and README proof/release sections against `openspec/specs/release-evidence/spec.md`, `openspec/specs/validation/spec.md`, `scripts/prove-self-hosting.sh`, `bootstrap/seed.ncl`, and the current bootstrap notes in `AGENTS.md`.

## Phase 2: Refresh the README operator workflow

- [x] Update `README.md` so the operator workflow covers `crunch doctor`, `crunch build --plan`, `crunch shell` / `crunch develop`, `crunch attest`, `crunch release`, and `--strict-hermetic`.
- [x] Update README build/reporting sections so they mention artifact attestation references and point readers at deeper docs when the top-level explanation would get too long.
- [x] Keep the README workflow-first by refreshing or adding focused-doc cross-links instead of dumping every flag and example inline.

## Phase 3: Refresh focused docs

- [x] Update `docs/benchmark-suite.md` so smoke, full-suite, and compare commands match the current benchmark tooling and sparse metric behavior.
- [x] Update `docs/bootstrap-stage0-inventory.md` and the corresponding README release/proof sections so release verification and bootstrap claim boundaries use the same bounded wording.
- [x] If a new focused doc page is needed for operator workflows, link it from `README.md` and keep its command examples aligned with current help output.

## Phase 4: Verify the documentation pass

- [x] Re-read the updated docs against `openspec/specs/cli/spec.md`, `openspec/specs/performance/spec.md`, `openspec/specs/release-evidence/spec.md`, and `openspec/specs/validation/spec.md`.
- [x] Verify the documented command surface against current help output for `crunch --help`, `crunch build --help`, `crunch shell --help`, `crunch attest --help`, and `crunch release --help`.
- [x] Run `openspec validate refresh-readme-and-operator-docs`.
