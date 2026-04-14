# Tasks: Normalize build execution envelope

## Phase 1: Canonical env

- [ ] Define the canonical build envelope for sandboxed builds
- [ ] Normalize reproducibility-sensitive env vars in the build-request path
- [ ] Decide which vars strict mode forbids derivations from overriding
- [ ] Add strict-mode validation for forbidden overrides

## Phase 2: Umask and tests

- [ ] Set an explicit build umask before the builder starts
- [ ] Add tests for locale, timezone, shell, and temp-dir normalization
- [ ] Add tests that host umask does not perturb builder-visible permissions

## Phase 3: Validation

- [ ] Re-read the touched build-pipeline spec against the final envelope shape before implementation starts
- [ ] Run `openspec validate normalize-build-execution-envelope`
