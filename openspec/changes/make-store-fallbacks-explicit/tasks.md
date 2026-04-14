# Tasks: Make store fallbacks explicit

## Phase 1: Persistent PathInfo policy

- [ ] Turn persistent `PathInfo` open failure into a typed hermeticity audit event
- [ ] Make strict mode fail instead of using in-memory `PathInfo`
- [ ] Keep practical mode usable with explicit degraded reporting

## Phase 2: Closure policy

- [ ] Turn degraded closure resolution into a typed hermeticity audit event
- [ ] Make strict mode fail when required source-input closure facts are missing
- [ ] Keep practical mode best-effort behavior with explicit degraded reporting

## Phase 3: Tests and validation

- [ ] Add regression tests for strict failure on both fallback paths
- [ ] Add regression tests for practical-mode reporting on both fallback paths
- [ ] Run `openspec validate make-store-fallbacks-explicit`
