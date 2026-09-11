# Specification: Relocatable dynamic outputs

## ADDED Requirements

### Requirement: Dependency references are origin-relative

r[mantle.relocatable_outputs.origin_relative_needed] A relativized dynamic
output MUST reference each dependency library through a path relative to the
referencing file, and that path MUST contain the dependency's store hash and
name literally.

The store is flat, so a dependency of a file at depth d inside an output is
`$ORIGIN/` plus `../` repeated (d+1) times plus the dependency's
`<hash>-<name>` directory. The loader MUST resolve the dependency without a
search over unrelated directories. The dependency's hash MUST remain a literal
substring so existing reference scanning finds it unchanged.

#### Scenario: Same references, relative spelling

- GIVEN a dynamic output with absolute NEEDED and RUNPATH entries and its
  relativized form
- WHEN the reference scanner processes both
- THEN both MUST yield the same reference set.

#### Scenario: Output runs from a copy

- GIVEN a relativized output whose closure is present beside the copy
- WHEN the copied binary runs
- THEN dynamic dependencies MUST resolve relative to the copy.

### Requirement: Bounded in-place fixup admission

r[mantle.relocatable_outputs.bounded_fixup_admission] The fixup MUST rewrite
existing bytes in place and MUST refuse any ELF shape it cannot prove safe.

Link steps that reserve rewrite capacity (RUNPATH padding, suffixed entries)
MUST be used where available. The fixup MUST detect and refuse: a dynamic
string table where a symbol name shares bytes with a rewrite target, a target
string without reserved capacity, an unsupported class or endianness, and a
truncated or corrupt image. Refusal MUST be a typed error naming the file and
reason; partial rewrites MUST NOT be published.

#### Scenario: Safe padded rewrite

- GIVEN a link output with reserved RUNPATH capacity for each dependency entry
- WHEN the fixup runs
- THEN NEEDED and RUNPATH entries become origin-relative in place and file
  length is unchanged.

#### Scenario: Symbol-tail overlap

- GIVEN an image where a symbol name is a byte suffix of the RUNPATH string
- WHEN the fixup runs
- THEN the fixup MUST refuse the file with a typed error and MUST NOT rewrite.

### Requirement: Launcher records replace wrapper scripts

r[mantle.relocatable_outputs.record_launchers] Wrapper outputs MUST use a
bounded launcher-record form: one static launcher binary per output family and
a typed record naming the program, fixed arguments, and environment defaults,
with store paths expressed relative to the output root.

Generated shell wrappers that capture build-time environment MUST NOT be
published by adopted families. Records MUST be bounded in size and count, and
the launcher MUST fail closed on a missing, malformed, or oversized record.

#### Scenario: Wrapper output launches through a record

- GIVEN a wrapper output migrated to the launcher-record form
- WHEN the launcher runs
- THEN it MUST exec the recorded program with the recorded arguments and
  environment, and argv zero MUST match the invoked name.

#### Scenario: Malformed record

- GIVEN a launcher invoked with a missing or malformed record
- WHEN it starts
- THEN it MUST fail with a typed error and MUST NOT execute anything.

### Requirement: Prefix independence is checkable

r[mantle.relocatable_outputs.prefix_independence] A relativized output MUST be
usable under any configured logical prefix without rebuilding.

Verification MUST run the output's declared check from a copy placed under a
second configured prefix with the closure present. The check composition
reuses the finish-gate relocation rerun where both are adopted.

#### Scenario: Second prefix without rebuild

- GIVEN a relativized output built under one logical prefix and materialized
  under a second configured prefix
- WHEN its declared check runs
- THEN it MUST pass without any rebuild.

#### Scenario: Absolute residue

- GIVEN an output that still contains an absolute reference to the build
  prefix
- WHEN prefix independence is checked
- THEN the check MUST fail naming the file and excerpt.

### Requirement: Reference scanning parity

r[mantle.relocatable_outputs.reference_scanning_parity] Relativization MUST
NOT change the reference set, closure membership, or garbage-collection roots
of an output.

The parity MUST hold for NEEDED entries, RUNPATH entries, launcher records,
and embedded configuration values that the contract admits.

#### Scenario: Closure membership unchanged

- GIVEN an output before and after relativization
- WHEN closure resolution runs from stored PathInfo
- THEN both closures MUST contain the same paths.

#### Scenario: Hash drift caught

- GIVEN a relativized output whose dependency entry names a wrong hash
- WHEN parity checking compares the reference set with the declared inputs
- THEN the mismatch MUST fail closed.
