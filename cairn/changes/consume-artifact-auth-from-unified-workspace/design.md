# Design: Unified Artifact source migration

## Context

Mantle consumes `artifact-auth-core` and `artifact-auth-ed25519` through exact Cargo and Nix pins. Artifact revision `c932138d880ddf4c2967f4c024b489b5c0022bf1` contains both packages with unchanged library entry bytes. The source workspace also contains transfer and binding packages.

## Decisions

### Use one immutable unified source

Cargo and Nix will identify the same Artifact repository and revision. The source check will require all four workspace members. Mantle's lock will admit only the two authentication packages.

### Keep validation logic pure

A pure Nix validator will receive parsed manifests, locks, source metadata, and resolved Nix identity. It will return deterministic issue codes. The flake shell will only load files, hash source entries, and fail on issues.

### Keep historical evidence immutable

The accepted Radicle receipt will remain unchanged. A new typed Nickel receipt will describe the active unified-source migration. The active check will not treat the historical receipt as current dependency state.

### Preserve behavior and authority

No Mantle Rust implementation will change. Existing positive and negative authentication tests remain the behavioral oracle. Mantle retains all product authority.

## Risks

- A wider source workspace can enter the consumer graph. The lock check will reject transfer or binding packages.
- Mixed Cargo and Nix sources can hide drift. The source validator will reject repository, revision, NAR, and package mismatches.
- Historical evidence can be mistaken for current state. The new receipt will mark the predecessor source as historical only.

## Rollback

Rollback is explicit. Restore the predecessor source, regenerate Cargo and Nix locks with their tools, rerun checks, and record a new receiver decision. No automatic fallback is allowed.
