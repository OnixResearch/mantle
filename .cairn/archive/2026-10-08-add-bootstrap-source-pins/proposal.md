# Proposal: Add bootstrap source pin tracking

## Why

Every `bootstrap/*.ncl` pins its upstream source inline with a URL and a hash.
Bumping a version means hand-editing Nickel source per file. The repository
record shows the recurring cost: upstream host moves handled one recipe at a
time (the `ftp.gnu.org` to `ftpmirror.gnu.org` migration), version bumps that
touch derivation code, and no way to ask "what is outdated?" without editing.

The reviewed external reference (`mic92/repkgs`, commit `1cd7b8b`, see
`evidence/repkgs-review.md`) records the researched lesson: machine-written
state lives in a data file next to the package; the build language only reads
it. Its `sources.toml` plus `uptrack` pipeline separates batched upstream
resolution, a pure decide stage emitting a plan, and an apply stage that
writes only pin data.

## What Changes

- Define a versioned, machine-writable pin record per bootstrap source:
  upstream identity (package-URL form), URL template, pinned version, and
  content hash, with validated fields and unknown-key rejection.
  r[mantle.bootstrap_source_pins.pin_data_contract]
- Make Nickel a reader only: bootstrap recipes read pin data; the updater
  never edits Nickel source, so a bump is one data-file write.
  r[mantle.bootstrap_source_pins.nickel_reads_only]
- Add a check command that resolves upstream releases in one batched, cached
  pass with per-host rate bounds and reports pending updates with a nonzero
  exit for automation. r[mantle.bootstrap_source_pins.batched_resolution]
- Add an apply command that consumes an unchanged plan, substitutes the
  version into URL templates, prefetches through the existing fixed-output
  fetch path, and writes only pin data.
  r[mantle.bootstrap_source_pins.apply_writes_pins_only]

## Impact

- **Immediate consumer**: Mantle bootstrap maintenance sessions and the
  `bootstrap/` recipe family; the `mantlepkgs-update-plans` family remains the
  owner of catalog-package updates, and this change stays on the bootstrap
  source family only.
- **Immediate outcome**: `mantle` gains "what is outdated" and "bump this
  pin" as data operations with no Nickel edits.
- **Durable capability**: upstream identity as data — one auditable pin file
  per source, batched polling, and a reviewable plan artifact.
- **Maintenance owner**: Mantle project/source-tooling owner, covering
  `crunch-project` and the new pin pipeline.
- **Repeatability evidence**: cached-resolution transcripts, plan JSON
  fixtures, apply diffs limited to pin data, and negative fixtures for
  unknown fields, missing hashes, and upstream mismatch.
- **Compatibility**: existing NCL inline pins keep working; recipes migrate
  family by family to pin data.

## Scope

The change covers the pin record format, the reader seam in bootstrap
recipes, the batched resolver with caching, the pure decide stage, the plan
artifact, the apply stage, and migration of the bootstrap recipe family.

## Out of Scope

- Catalog-package update planning (owned by the accepted
  `mantlepkgs-update-plans` spec family).
- Lock-file-driven dependency fetching (owned by
  `add-lock-driven-vendor-fetches`).
- Fetching anything at evaluation time; evaluation stays pure.
- Automatic unattended upgrades or advisory scanning.

## Success Criteria

- `check` on an unchanged world performs only conditional cached requests and
  exits zero; pending updates exit nonzero and name them.
- `apply` writes pin data files only; the working tree diff shows no Nickel
  source edits.
- A recipe whose pin data names an unknown field fails validation with the
  field listed.
- Migration of a recipe family needs no change to its build phases.
