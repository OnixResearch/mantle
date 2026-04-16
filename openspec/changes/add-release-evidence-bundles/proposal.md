# Add release evidence bundles

## Why

The repo now has a self-hosting proof and a clearer bootstrap maturity story,
but it still does not ship a release artifact that another operator can inspect
and verify without reading a long session transcript. That leaves a gap between
"the project can prove this locally" and "a release carries the evidence needed
for someone else to trust what it claims."

A release evidence bundle closes that gap. It does not magically prove full
source bootstrap or global reproducibility, but it does package the exact
artifacts, digests, and proof references a reviewer needs to verify a release
claim.

## What Changes

- define a release evidence bundle format for crunch releases
- add commands to build and verify those bundles using bundle-local inputs only
- package the release binary or binaries, staged-source archive, full proof
  bundle output, and prerequisite inventory into one manifest-driven artifact
- bind the proof bundle and prerequisite inventory to the exact release
  identifier, source digest, and binary digests recorded in the manifest
- reject prerequisite-only checks as proof artifacts for release evidence
- document release evidence as distinct from full-source bootstrap claims

## Capabilities

### New Capabilities
- `release-evidence-bundle`: produce a manifest-driven artifact for a release
  candidate
- `bundle-verification`: verify the bundle contents and embedded digests later
- `proof-linked-manifest`: connect a release candidate to the proof bundle and
  prerequisite inventory used to justify it

## Impact

- **Files**: CLI release commands, release-manifest types, bundle docs,
  bootstrap or README docs on claim boundaries
- **APIs**: manifest generation and verification helpers
- **Dependencies**: optional archive packaging helpers only if needed
- **Testing**: bundle creation, bundle verification, missing-artifact failure,
  digest mismatch failure, and documentation alignment checks

## Non-Goals

- claiming full-source bootstrap because a release bundle exists
- remote signing infrastructure in this first change
- independent rebuild orchestration by the verifier command itself
- hiding the proof prerequisites or trust inventory
