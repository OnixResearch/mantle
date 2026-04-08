## Phase 1: Proof workflow

- [x] Add one checked-in helper that sets the required build environment and runs `cargo test -p crunch --test self_hosting -- --ignored --nocapture`
- [x] Make the helper fail fast on missing host prerequisites or missing environment inputs
- [x] Update `README.md` so the helper is the default documented way to run the self-hosting proof

## Phase 2: Main spec sync

- [x] Update `openspec/specs/bootstrap/spec.md` to require the checked-in proof entry point
- [x] Update `openspec/specs/build-pipeline/spec.md` so bounded concurrent builds are documented as current behavior
- [x] Update `openspec/specs/portability/spec.md` so current Linux-only build support is explicit and future builders stay labeled as future work

## Phase 3: Verification

- [x] Run the checked-in proof entry point far enough to verify the invocation path and prerequisite checks
- [x] Run `openspec validate proof-workflow-and-spec-sync`
- [x] Re-read the touched README / spec sections against the current code and help text before landing the change
