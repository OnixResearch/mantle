# Tasks: Make store fallbacks explicit

## Phase 1: Persistent PathInfo policy

- [x] Turn persistent `PathInfo` open failure into a typed hermeticity audit event
- [x] Make strict mode fail instead of using in-memory `PathInfo`
- [x] Keep practical mode usable with explicit degraded reporting

## Phase 2: Closure policy

- [x] Turn degraded closure resolution into a typed hermeticity audit event
- [x] Make strict mode fail when required source-input closure facts are missing
- [x] Keep practical mode best-effort behavior with explicit degraded reporting

## Phase 3: Tests and validation

- [x] Add regression tests for strict failure on both fallback paths
- [x] Add regression tests for practical-mode reporting on both fallback paths
- [x] Run `openspec validate make-store-fallbacks-explicit`
