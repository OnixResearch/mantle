## MODIFIED Requirements

### Requirement: Native closure resolution

The system MUST resolve runtime closures using its own data, not by shelling
out to `nix-store -qR`.

For each source input that needs closure resolution:

1. Check local `PathInfo` for the `references` field
2. If not in local `PathInfo`, query remote narinfo data
3. Walk references transitively until the full closure is collected
4. If neither source is available:
   - strict hermetic mode MUST fail before sandbox start,
   - practical mode MAY mount the declared path only, but it MUST emit a
     degraded closure audit event.

#### Scenario: Strict mode rejects missing closure data

- GIVEN a strict build that depends on a source input with no local `PathInfo` and no narinfo
- WHEN closure resolution runs
- THEN the build fails before sandbox start
- AND the error identifies the missing closure facts for that source input

#### Scenario: Practical mode records degraded closure data

- GIVEN a practical build that depends on the same source input
- WHEN closure resolution runs
- THEN crunch mounts only the declared path
- AND the build result records a degraded closure audit event

## ADDED Requirements

### Requirement: Persistent PathInfo fallback policy is explicit

The store layer MUST treat failure to open persistent `PathInfo` state as an
explicit execution mode change, not an invisible recovery.

Practical mode MAY continue with in-memory `PathInfo` only if the degraded mode
is reported to the caller. Strict hermetic mode MUST fail immediately instead of
falling back to in-memory state.

#### Scenario: Practical mode uses reported in-memory fallback

- GIVEN the persistent `PathInfo` database cannot be opened
- WHEN a practical build starts
- THEN crunch may continue with in-memory `PathInfo`
- AND the build result records a degraded pathinfo audit event

#### Scenario: Strict mode rejects in-memory fallback

- GIVEN the persistent `PathInfo` database cannot be opened
- WHEN a strict build starts
- THEN crunch fails before building
- AND the error says strict mode does not permit in-memory `PathInfo` fallback
