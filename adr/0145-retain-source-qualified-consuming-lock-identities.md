# ADR 0145: Retain source-qualified consuming-lock package identities

## Status

Accepted (2026-11-06) for bounded Rust source topology construction. This does
not accept source preparation, provider execution, a fixed point or a release.

## Context

After the compact native continuation repair, unchanged-pin source preparation
rejected `ambiguous original locked identity home`. The Rust 1.93.1 and 1.94.1
Cargo workspaces each lock both a local and a registry `home` at version 0.5.12.
Their Cargo dependency explicitly names
`home 0.5.12 (registry+https://github.com/rust-lang/crates.io-index)`.
The resolver discarded that source qualifier and compared only name/version.

Merely filtering that edge by source would still leave an authority error:
transitive resolution selected the first owner with a matching name/version.
A local record could therefore supply a registry package's children. A behavior
regression with distinguishable children exposed that error independently of
the original ambiguity rejection.

Original Cargo's lockfile decoder is the relevant source, not its general
PackageIdSpec query matcher. The original lock decoder admits version before
source, prefers a unique path identity when source is omitted, and otherwise
requires a unique candidate. An isolated offline host Cargo metadata probe
confirmed local/registry coexistence and distinct transitive children, plus a
unique-registry control where omitted source is valid. That host diagnostic is
not protected bootstrap execution.

## Decision Drivers

- Preserve the single borrowed consuming lock/patch authority from ADR 0141.
- Preserve name, optional version and optional source qualification together.
- Do not choose by table order, drop source text or substitute dependency locks.
- Preserve the selected owner's children through feature accumulation/revisits.
- Keep exported manifest/host keys and consumer interfaces unchanged.
- Retain fail-closed ambiguity, missing identities and bounded source authority.

## Decision

Parse the original lock dependency spelling as name, optional version and
optional parenthesized source. Consume the complete spelling. A supplied source
requires a supported Cargo source kind and a valid URL; exact serialized source
matching never falls back to another source. The original admitted cohort uses
canonical serialized identities; this is not a general URL-alias resolver.

First admit the requested version. Omitting a version requires a single version
for that name. Within that version, explicit source requires its unique exact
identity. With no source qualifier, use a unique local/path record if present;
otherwise require one remaining identity. Selected duplicate identities,
unmatched qualification and unresolved ambiguity remain errors.

Carry the exact borrowed lock record selected for an external dependency through
the resolver worklist and feature-request state. Its own locked children must be
resolved from that record, not another same-name/version record. Local/path or
patch selections retain their separate authority and use local-owner lookup.
Check owner identity before feature deduplication so a repeated request cannot
silently merge different authorities behind one manifest/host key.

No lockfile is cloned into a key or package, and no exported key shape grows.
The consuming lock remains shared and immutable across transitive traversal;
compiler, standard-library and Cargo groups remain separate resolutions. No
version order, dependency-local lock fallback, source-profile substitution,
network fetch or resource-limit change is introduced.

## Consequences

The current consumers are Cargo-free successor compiler/tool source topologies,
particularly Cargo in Rust 1.93.1 and 1.94.1. Mantle's Rust source-authoring
owners maintain the lookup and borrowed-owner state. Repeatability evidence
combines original archive/lock facts, failed-before/passed-after consumer
regressions, original Cargo decoder inspection, an explicitly offline host
metadata diagnostic and a separate unchanged-pin real preparation attempt.
These observations do not establish a general Cargo resolver, compiler
correctness, provider construction, full-source fixed point or release readiness.
