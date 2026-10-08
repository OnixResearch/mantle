# Specification: Bootstrap source pins

## ADDED Requirements

### Requirement: Pin data contract

r[mantle.bootstrap_source_pins.pin_data_contract] Each bootstrap source MUST
be pinned by a versioned machine-writable data record holding an upstream
identity in package-URL form, one or more URL templates keyed by role, the
pinned version with release date, and the content hash of each fetched
artifact.

The record format MUST reject unknown keys and fields with an error naming
the known ones. Hash format MUST distinguish flat and unpacked-tree hashes
consistently with the existing fixed-output fetch contract.

TOML under `bootstrap/pins/` is the authoritative record. Its declared
`recipes` name the migrated Nickel consumers; `artifacts.<role>.url` MUST
equal substitution of `version` in `url_template`. A derived JSON projection
under `bootstrap/pins/generated/` MUST match the TOML record exactly in the
deterministic reader format before check or apply.

#### Scenario: Valid record round-trips

- GIVEN a well-formed pin record for a bootstrap source
- WHEN validation and the reader seam process it
- THEN the record MUST validate and the recipe MUST resolve the same URL and
  hash as the previous inline pin.

#### Scenario: Unknown field

- GIVEN a pin record with a field outside the contract
- WHEN validation runs
- THEN Mantle MUST fail listing the unknown field and the known fields.

### Requirement: Nickel reads pins only

r[mantle.bootstrap_source_pins.nickel_reads_only] Bootstrap recipes MUST
obtain upstream URL, version, and hash by reading pin data; the updater MUST
NOT edit Nickel source.

Changing a pin MUST be exactly one data-file write plus regeneration of any
declared derived data. Build logic in `bootstrap/*.ncl` MUST NOT depend on
fields the pin record does not carry.

The bounded repository migration validation rail MUST export each declared
Nickel recipe and reject an evaluated fixed-output fetch whose URL, hash, or
flat/tree mode differs from the authoritative pin. Runtime check/apply MUST
verify pin and derived-reader bytes without requiring a standalone Nickel
binary; they MUST NOT claim to have semantically checked recipe binding.

#### Scenario: Version bump without recipe edits

- GIVEN a recipe family migrated to pin data and a new upstream release
- WHEN apply writes the new pin
- THEN the recipe tree MUST remain unmodified and the rebuild MUST use the
  new URL and hash.

#### Scenario: Recipe retains an old upstream URL after a pin change

- GIVEN a migrated recipe hardcodes its previous upstream URL while the pin
  and derived reader declare a newer release
- WHEN the repository semantic recipe validation rail evaluates the recipe
- THEN the evaluated fetch MUST be rejected as an incomplete migration.

### Requirement: Batched resolution with caching

r[mantle.bootstrap_source_pins.batched_resolution] The check command MUST
resolve upstream releases for all pinned sources in one batched pass with
per-host request bounds, conditional-request caching, and no per-source
custom code unless a source declares a resolve hook.

An unchanged upstream MUST be answered from cache without parsing a new body.
The check result MUST report current, candidate, and error states per source
and MUST exit nonzero when updates are pending or resolution failed.

#### Scenario: Unchanged world

- GIVEN all upstreams unchanged since the last cached pass
- WHEN check runs
- THEN every source MUST be answered from cache and the exit MUST be zero.

#### Scenario: New upstream release

- GIVEN one source with a newer upstream release
- WHEN check runs
- THEN the report MUST name the source with current and candidate versions
  and the exit MUST be nonzero.

### Requirement: Apply writes pin data only

r[mantle.bootstrap_source_pins.apply_writes_pins_only] The apply command MUST
consume an unchanged or explicitly edited plan artifact, substitute the pinned
version into URL templates, prefetch each artifact through the existing
fixed-output fetch path, and write only pin data files.

Apply MUST fail closed when any plan entry has an unresolved upstream error,
when a fetch hash does not match the upstream-observed hash, when a plan entry
names an unknown source, or when the plan was mutated without review. Apply
MUST be idempotent for already-current pins.

An explicitly edited plan MUST be resealed and reviewed before apply; an
unresealed edit or changed TOML preimage MUST fail closed. Apply MUST stage
the TOML and derived-reader replacements and hold the source directory
mutation lock through verification and publication; an external noncooperating
editor is not serialized by this advisory lock.

#### Scenario: Clean apply

- GIVEN a reviewed plan with one pending update
- WHEN apply runs
- THEN exactly that source's pin data MUST change, the fetch MUST go through
  the fixed-output path, and the recorded hash MUST match the upstream
  observation.

#### Scenario: Hash mismatch

- GIVEN an upstream whose fetched artifact hash differs from the plan's
  recorded hash
- WHEN apply runs
- THEN apply MUST fail closed, write nothing, and name the source.

#### Scenario: Mixed candidate and unresolved source

- GIVEN a sealed plan with one candidate and one upstream resolution error
- WHEN reviewed apply runs
- THEN apply MUST reject the whole plan before downloading or writing any
  TOML or derived reader and MUST name the unresolved source.
