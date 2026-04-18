## ADDED Requirements

### Requirement: Lazy session APIs return typed errors instead of panicking

The `crunch-eval` lazy session API MUST return `crunch-eval::Error` values for
malformed or inconsistent top-level and selected-root shapes that can arise
from evaluated input or stale session state.

Public lazy-eval methods MUST NOT rely on `expect(...)` or `unwrap(...)` for
user-reachable shape checks.

Internal impossible-state assertions MAY remain as private invariants only
after user-reachable failures have already been converted into typed errors.

#### Scenario: Selected-root forcing rejects inconsistent shape without panic

- GIVEN a lazy-eval session whose selected-root extraction encounters a shape
  inconsistent with the discovered top-level structure
- WHEN a caller forces that root through the public lazy session API
- THEN the method returns a `crunch-eval::Error`
- AND the process does not panic

#### Scenario: Root discovery rejects malformed record or array shape without panic

- GIVEN a lazy-eval session whose discovery path encounters a malformed record
  or array structure for the current root shape
- WHEN a caller asks for root labels through the public API
- THEN the method returns a `crunch-eval::Error`
- AND the process does not panic
