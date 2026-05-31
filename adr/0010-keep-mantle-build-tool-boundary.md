# ADR 0010: Keep Mantle's boundary build-shaped

## Status

Accepted (2026-05-30)

## Context

Nix, NixOS, and nixpkgs are difficult to evolve independently because the build
tool, module system, package set, and OS configuration layer share too much
surface area. Mantle needs to avoid repeating that coupling. Mantle can be used
by higher-level frontends such as Onix, but it should not become the place where
Onix inventory semantics, module ABI, role expansion, settings contracts,
provider topology, package policy, or OS configuration lowering live.

A prior in-tree `mantle system eval` / `crunch-system` scaffold blurred that
line. It made Mantle look like a module evaluator even though the real module
semantics belong above Mantle.

## Decision

Mantle will remain a frontend-neutral build tool. Its stable boundary is build
shaped: evaluated derivations, build plans, source inputs, store operations,
build reports, and diagnostics about those build operations.

Onix and other frontends must own their module layer. They may evaluate modules,
validate settings, expand roles/tags, resolve provider topology, and choose
packages or artifacts, but they must lower that state before invoking Mantle.
Mantle may receive concrete build inputs or opaque data produced by a frontend;
it must not grow first-class Onix/NixOS-style module semantics in core CLI,
stdlib, workspace crates, or public examples.

Persistent tests must guard this boundary. If a future change needs a module
layer, it belongs in Onix or another adapter above Mantle unless a new ADR
explicitly supersedes this decision.

## Consequences

Mantle stays smaller and easier to reason about as a build engine. Frontends can
evolve module semantics without changing Mantle core. The integration seam is
less convenient than a single monolithic stack, but it avoids the Nix/NixOS/nixpkgs
coupling pattern.

Some tests will reject convenient shortcuts such as reintroducing a `system`
subcommand, exporting module contracts from Mantle's stdlib, or adding
NixOS-style module evaluators to the implementation surface.

## Alternatives Considered

### Rebuild the module layer inside Mantle

Rejected. It couples Mantle to one frontend's configuration model and repeats the
Nix/NixOS split problem inside a new codebase.

### Keep the system scaffold as experimental

Rejected for now. Even an experimental public surface invites users and tests to
depend on it as an integration contract.

### Let every frontend add its own Mantle subcommand

Rejected. Frontend-specific CLIs should live in those frontend repos or adapters,
not in Mantle core.
