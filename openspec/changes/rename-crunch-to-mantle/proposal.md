## Why

The project name `crunch` was useful during early bootstrap work, but the replacement-for-Nix system now needs a durable identity before broader release, documentation, and proof artifacts hard-code more user-facing names. `Mantle` better communicates a load-bearing substrate beneath evaluator, store, builder, and release-proof workflows.

## What Changes

- **Project identity**: Rename the canonical project, docs, package metadata, release evidence wording, and user-facing references from Crunch/crunch to Mantle/mantle.
- **CLI surface**: Make `mantle` the canonical binary and command spelling. Any `crunch` compatibility entry point must be explicit, tested, and described as transitional.
- **On-disk defaults**: Rename project-local files/directories and the default logical store prefix where doing so changes user-visible identity, with deliberate migration/compatibility behavior.
- **Proof and release artifacts**: Ensure self-build, attestation, release, and parity evidence identify the canonical product as Mantle without invalidating the narrow proof claims already specified.

## Capabilities

### New Capabilities
- `project-identity`: Canonical naming, compatibility aliases, and migration behavior for the Mantle rename.

### Modified Capabilities
- `cli`: Command names, help text, README command coverage, and transitional alias behavior.
- `project-management`: Manifest, lockfile, and generated-input filenames or compatibility handling.
- `store-prefix`: Default logical store prefix and compatibility with older `/crunch/store` artifacts.
- `release-evidence`: Product identity embedded in release/proof manifests and documentation.

## Impact

- **Files**: Cargo package metadata, CLI dispatch/help, docs/README, scripts, OpenSpec prose, bootstrap evidence receipts, release/proof manifests, project-management defaults, tests/fixtures.
- **APIs**: User-facing CLI changes from `crunch` to `mantle`; internal Rust crate/module renames are implementation details unless exposed in artifacts.
- **Data compatibility**: Existing `crunch.lock`, `crunch-project.ncl`, `.crunch/`, and `/crunch/store` references need either migration or a documented compatibility window.
- **Testing**: Verify `mantle --help`, command aliases, default paths, project migration, release/proof identity, and grep-level absence of unintended Crunch/crunch branding in user-facing surfaces.

## Out of Scope

- Changing core build semantics beyond name/default-path migrations.
- Rebranding unrelated upstream concepts such as Nix compatibility flags.
- Claiming stronger bootstrap, parity, or reproducibility guarantees as part of the rename.
