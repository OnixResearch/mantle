# Release Provenance Delta: Chaptered Transport

## ADDED Requirements

### Requirement: Chaptered release transport is optional

r[mantle.release_provenance.chapter_transport.optional] Mantle MUST keep the verified release directory and canonical `manifest.json` as release evidence authority, and it MAY provide a versioned chaptered `.tar.gz` transport as an opt-in container.

#### Scenario: Packing does not replace canonical evidence

- GIVEN a release evidence directory passes normal Mantle release verification
- WHEN an operator packs it as chaptered transport
- THEN Mantle MUST leave the source directory and canonical manifest unchanged
- AND the transport receipt MUST state that the archive is a container, not new release authority.

#### Scenario: Fixed formats remain unchanged

- GIVEN an OCI layer, Android image archive, NAR, upstream source archive, or existing release source tar has a fixed compatibility contract
- WHEN chaptered release transport is available
- THEN Mantle MUST NOT silently replace that format with chaptered transport.

### Requirement: Chapter planning is pure, deterministic, and bounded

r[mantle.release_provenance.chapter_transport.plan] Mantle MUST compute chapter grouping, member order, index fields, receipt fields, named limits, and deterministic diagnostics in `crunch-release-core` without filesystem, environment, process, clock, network, compression, or output effects.

#### Scenario: Equivalent trees produce one plan

- GIVEN two normalized release tree observations contain the same paths, kinds, modes, sizes, and link targets in different input orders
- WHEN the chapter planner evaluates them
- THEN it MUST produce identical ordered chapters and canonical index bytes
- AND it MUST assign the manifest to chapter zero, source files by source group, each binary by immediate binary child, and remaining members by top-level group.

#### Scenario: Invalid observations fail before output

- GIVEN observations contain a missing manifest, duplicate path, reserved index path, unsupported entry kind, unsafe path, invalid link, excessive member count, excessive chapter count, or byte-limit overflow
- WHEN planning runs
- THEN it MUST return deterministic ordered blockers
- AND no archive or receipt destination MAY become visible.

### Requirement: Pack output is deterministic and standard-compatible

r[mantle.release_provenance.chapter_transport.pack] `mantle release transport pack` MUST verify the source release bundle, write deterministic tar headers and chapter order, produce an ordinary-compatible gzip and tar stream, validate a complete staged round trip, and publish the archive plus canonical detached receipt without replacement.

#### Scenario: Valid bundle packs reproducibly

- GIVEN the same verified release tree, compression policy, and transport version
- WHEN Mantle packs it twice
- THEN both archives and receipts MUST be byte-identical
- AND ordinary gzip and tar readers MUST enumerate the reserved transport index plus every original bundle member.

#### Scenario: Source drift fails closed

- GIVEN a planned source file, directory, link, mode, size, or link target changes before its archive entry is written
- WHEN packing revalidates the source
- THEN packing MUST fail before destination publication
- AND neither final archive nor final receipt MAY contain partial output.

### Requirement: Detached receipt binds compressed transport bytes

r[mantle.release_provenance.chapter_transport.receipt] The transport receipt MUST bind the compressed archive BLAKE3, archive byte size, source manifest BLAKE3, transport-index BLAKE3, chapter count, member count, format version, `chapter-tgz` version, claim scope, and non-claims.

#### Scenario: Matching receipt permits inspection

- GIVEN an archive and canonical receipt have matching compressed-byte, manifest, index, count, and version fields
- WHEN Mantle inspects the transport
- THEN it MUST report the deterministic chapter index and bounded transport facts.

#### Scenario: Truncation or replacement is rejected

- GIVEN an archive is truncated, has one modified compressed byte, is paired with a receipt for another archive, has a rewritten receipt for malformed gzip bytes, or declares excessive chapter markers
- WHEN inspect or unpack measures the compressed bytes
- THEN it MUST check the complete bounded gzip stream and marker count
- AND it MUST reject invalid input before chapter access or destination mutation.

### Requirement: Inspect requires the versioned transport index

r[mantle.release_provenance.chapter_transport.inspect] `mantle release transport inspect` MUST require a matching receipt and a supported reserved index in chapter zero, compare every indexed member against archive metadata, and reject an ordinary one-chapter tgz that lacks the index.

#### Scenario: Chapter zero supports bounded inspection

- GIVEN a valid chaptered release transport and matching receipt
- WHEN inspect opens chapter zero
- THEN it MUST validate the reserved index and canonical release manifest without unpacking later chapter payloads
- AND it MUST report chapter roles, ordinals, members, and compressed sizes.

#### Scenario: Legacy tgz remains readable but is not admitted

- GIVEN `chapter-tgz` can expose an ordinary tgz as one chapter
- WHEN Mantle transport inspection finds no supported reserved index
- THEN it MUST return a deterministic unsupported-transport diagnostic
- AND it MUST NOT classify the ordinary tgz as a valid Mantle chapter transport.

### Requirement: Unpack is staged and capability-confined

r[mantle.release_provenance.chapter_transport.unpack] `mantle release transport unpack` MUST validate the complete indexed archive before mutation, extract supported entries through capability-relative no-follow operations into a private sibling stage, run normal release verification there, remove transport-only metadata, and publish the directory with an atomic no-replace rename.

#### Scenario: Valid transport recreates canonical directory

- GIVEN a valid archive, matching receipt, absent destination, and supported files, directories, and internal relative links
- WHEN Mantle unpacks the transport
- THEN the published directory MUST match the source release tree identity
- AND normal release verification MUST pass on the published directory.

#### Scenario: Unsafe archive fails without external writes

- GIVEN an archive contains traversal, absolute path, duplicate member, unsupported special type, privileged mode bits, escaping link, link-parent substitution, stale index metadata, oversized payload, or an existing destination
- WHEN Mantle validates or unpacks it
- THEN it MUST fail closed without following the unsafe entry
- AND paths outside the private stage and any competing destination MUST remain unchanged.

### Requirement: Chapter transport validation and non-claims are recorded

r[mantle.release_provenance.chapter_transport.validation] The change MUST include positive and negative core, shell, CLI, compatibility, and lifecycle evidence before archive, and documentation MUST preserve the transport-only claim boundary.

#### Scenario: Focused validation covers pass and fail paths

- GIVEN deterministic, standard-reader, round-trip, random-access, legacy, truncation, tamper, bounds, path, link, no-clobber, and representative benchmark fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed
- AND evidence MUST record exact commands, dependency identity, sequential and parallel measurements, known maturity limits, and any blocked broad checks.

#### Scenario: Transport evidence does not strengthen release claims

- GIVEN a chaptered archive and receipt validate
- WHEN Mantle renders output or documentation
- THEN it MUST limit the result to compressed transport identity, index consistency, safe materialization, and preserved release-bundle verification
- AND it MUST NOT claim build correctness, source correctness, semantic correctness, reproducibility, deployment safety, or universal release eligibility.
