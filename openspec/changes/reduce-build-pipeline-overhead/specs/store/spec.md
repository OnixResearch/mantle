# Store Crate Specification

## Purpose

Defines the `crunch-store` crate: a standalone module owning all
store operations — service construction, caching, realization, queries,
and CA mapping persistence.

## MODIFIED Requirements

### Requirement: Native closure resolution

The system MUST resolve runtime closures using its own data, not by shelling
out to `nix-store -qR`.

For each source input that needs closure resolution:

1. check local `PathInfo` for the `references` field,
2. if not in local `PathInfo`, fetch and parse remote narinfo metadata for
   `References:`,
3. walk references transitively until the full closure is collected,
4. download and ingest a remote NAR only when actual substitution is
   requested, not while enumerating closure references,
5. if neither source is available, use the declared path only and log a
   warning.

#### Scenario: Crunch-built input closure from PathInfo

- GIVEN a crunch-built library `libfoo` with `PathInfo` in redb
- AND `PathInfo.references` lists `libbar` and `glibc`
- WHEN a derivation depends on `libfoo`
- THEN `libfoo`, `libbar`, and `glibc` are all mounted in the sandbox

#### Scenario: Nix seed input closure from narinfo metadata

- GIVEN a Nix seed path `/nix/store/...-bash-5.2`
- AND the remote narinfo for that path lists 3 references
- WHEN a derivation depends on bash
- THEN bash and its 3 transitive references are mounted
- AND crunch does not download the bash NAR payload merely to continue the
  closure walk

#### Scenario: No closure data available

- GIVEN a seed path with no local `PathInfo` and no remote narinfo
- WHEN a derivation depends on it
- THEN only the declared path is mounted
- AND a warning is logged

#### Scenario: Static input needs no closure

- GIVEN a statically-linked binary from `crunch bootstrap --fetch`
- WHEN a derivation depends on it
- THEN `PathInfo.references` is empty
- AND only the declared path is mounted
