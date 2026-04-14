# Tasks: Normalize build execution envelope

## Phase 1: Canonical env

- [x] Define the canonical build envelope for sandboxed builds
- [x] Normalize reproducibility-sensitive env vars in the build-request path
- [x] Decide which vars strict mode forbids derivations from overriding
- [x] Add strict-mode validation for forbidden overrides

## Phase 2: Umask and tests

- [x] Set an explicit build umask before the builder starts
- [x] Add tests for locale, timezone, shell, and temp-dir normalization
- [x] Add tests that host umask does not perturb builder-visible permissions

## Phase 3: Validation

- [x] Re-read the touched build-pipeline spec against the final envelope shape before closing the change
- [x] Run `openspec validate normalize-build-execution-envelope`
