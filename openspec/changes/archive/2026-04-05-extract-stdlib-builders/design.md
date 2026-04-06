## Context

`lib/mk_derivation.ncl` implements a full phase-based build system
(unpack, configure, build, install, fixup, check) with mkStdenv,
mkDerivation, mkShell, callPackage, and overrideAttrs. The stdlib
spec says core should ship the schema, not the opinions. This code
grew into core during bootstrap development.

## Goals / Non-Goals

**Goals:** Move builder logic out of the core stdlib. Re-close the
Derivation contract. Keep bootstrap working.

**Non-Goals:** Redesign mkDerivation. Add new phases. Change
overrideAttrs semantics.

## Decisions

### 1. New `builders/` directory at the crunch repo root

**Choice:** `builders/lib.ncl` as the entry point. Contains
mkDerivation, mkStdenv, mkShell, callPackage, MkDerivationArgs.

**Rationale:** It's a Nickel package that imports the crunch stdlib.
Living in the same repo is fine for now — it can be split into its
own repo later. The important thing is the import boundary: builders
imports lib, not the other way around.

**Alternative:** `packages/stdenv/`. Rejected — too deep, and "stdenv"
is Nix jargon. "builders" is clearer.

### 2. Close the Derivation contract

**Choice:** Remove `..` from `lib/derivation.ncl`. The contract
becomes a closed record with exactly the fields the glue layer
consumes.

**Rationale:** Field name typos should be caught at eval time. The
`..` was only needed because mkDerivation passed pname/version/meta/
passthru through to the final record. After extraction, mkDerivation
produces a clean Derivation record without those fields.

**Risk:** Any user .ncl files that add extra fields to Derivation
records will break. This is the correct behavior — extra fields were
silently dropped before.

### 3. Bootstrap .ncl files import from builders/

**Choice:** Files in `bootstrap/` change their import from
`let crunch = import "lib.ncl"` to also import
`let builders = import "builders/lib.ncl"`.

**Rationale:** Bootstrap files use mkDerivation-style phases. They
need the builder package. The crunch binary's `--import-path` already
supports pointing at multiple directories.

### 4. Order: independent of other changes

**Choice:** This change can land independently. It's a pure Nickel
refactor with no Rust code changes.

**Rationale:** The Derivation contract, the lib.ncl exports, and the
.ncl import paths all change. But no Rust types, no build pipeline,
no store logic changes.

## Risks / Trade-offs

**[Breaking user .ncl files]** Any file that uses `crunch.mkDerivation`
directly must change to import the builder package. Mitigated by the
fact that crunch has no external users yet.

**[Closed contract strictness]** Users who added custom metadata fields
to Derivation records need to move them to `env` or use the builder
package's open contract. This is the intended design.
