# Closure Resolution Specification

## Purpose

Defines how crunch resolves runtime closures for store path inputs
without depending on the `nix-store` binary.

## ADDED Requirements

### Requirement: Native closure resolution

The system MUST resolve runtime closures using its own data, not
by shelling out to `nix-store -qR`.

For each source input that needs closure resolution:

1. Check local PathInfo (redb) for the `references` field
2. If not in local PathInfo, query the binary cache narinfo for
   `References:`
3. Walk references transitively until the full closure is collected
4. If neither source is available, use the declared path only
   (no closure) and log a warning

#### Scenario: Crunch-built input closure from PathInfo

- GIVEN a crunch-built library `libfoo` with PathInfo in redb
- AND PathInfo.references lists `libbar` and `glibc`
- WHEN a derivation depends on `libfoo`
- THEN `libfoo`, `libbar`, and `glibc` are all mounted in the sandbox

#### Scenario: Nix seed input closure from narinfo

- GIVEN a Nix seed path `/nix/store/...-bash-5.2`
- AND the binary cache narinfo for that path lists 3 references
- WHEN a derivation depends on bash
- THEN bash and its 3 transitive references are mounted

#### Scenario: No closure data available

- GIVEN a seed path with no local PathInfo and no narinfo
- WHEN a derivation depends on it
- THEN only the declared path is mounted
- AND a warning is logged: "no closure data for /nix/store/..., mounting without closure"

#### Scenario: Static input needs no closure

- GIVEN a statically-linked binary from `crunch bootstrap --fetch`
- WHEN a derivation depends on it
- THEN PathInfo.references is empty
- AND only the declared path is mounted (correct behavior)

### Requirement: Cycle-safe closure walking

The closure walker MUST detect cycles in the reference graph and
terminate. Store paths with circular references MUST NOT cause
infinite loops.

#### Scenario: Circular reference

- GIVEN path A references B and B references A
- WHEN closure resolution runs for A
- THEN both A and B are included, and the walk terminates

### Requirement: Closure depth limit

The closure walker MUST enforce a maximum transitive depth (1024).
Paths beyond this depth MUST be skipped with a warning.

#### Scenario: Deep closure

- GIVEN a path with a transitive closure deeper than 1024
- WHEN closure resolution runs
- THEN the first 1024 levels are included
- AND a warning is logged about the depth limit

## REMOVED Requirements

### nix-store subprocess call

The `resolve_nix_closure()` function and its `nix-store -qR` subprocess
call MUST be removed. The system MUST NOT shell out to any Nix tool
for core functionality.

#### Scenario: Build without nix-store

- GIVEN a machine with crunch but no Nix installed
- AND seed inputs have narinfo available via binary cache
- WHEN `crunch build` runs
- THEN the build succeeds (closures resolved from narinfo)
- AND no subprocess to `nix-store` is attempted
