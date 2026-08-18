# Proposal: Offline source bundle manifest

## Summary

Add a Mantle-native offline source bundle for air-gapped and unreliable-network builds. Operators should be able to plan, export, list, import, and verify the source/input material needed by any build before realization starts: fixed fetcher inputs, local source trees, VCS snapshots, package-manager mirrors, bootstrap source archives, provider manifests, toolchain/source-root inputs, and proof inputs. This complements store-output archives; it moves declared inputs to the offline host before any build or remote dispatch claim.

## Motivation

`store-archive-transport` covers produced output closures, and `p2p-remote-builders` covers builder-side missing-input upload during a live remote session. The remaining offline gap is earlier: an operator needs to prove that a disconnected machine already has the source and provider input closure required to build, without relying on ambient network access, language package-manager caches, sibling checkouts, or a remote builder session.

Mantle already records explicit source closure facts across fetchers, project inputs, bootstrap/provider flows, and language-specific planners such as Rust. Those facts should become a portable, inspectable bundle with BLAKE3 identities, source-kind metadata, and fail-closed verification. The bundle is not a binary cache and not a build-output archive; it is the declared input/source side of an offline build handoff.

## Scope

- Define a versioned Mantle source-bundle manifest with deterministic source records, BLAKE3 content refs, source-kind metadata, logical store prefix when store paths appear, and bounded record/payload limits.
- Define a language-neutral source adapter contract for package managers and build ecosystems: lock identity, offline knobs, allowed source roots, cache isolation, generated-source boundaries, and fail-closed unsupported behavior.
- Canonicalize filesystem source payloads with explicit path, symlink, mode, hardlink, device, timestamp, Unicode, case-sensitivity, and traversal rules.
- Add `mantle source bundle plan|export|list|import|verify` or equivalent API surfaces.
- Support declared fetcher inputs, local source trees, VCS checkout snapshots, language package-manager mirrors such as Cargo vendor roots, npm/pnpm stores, Go module mirrors, Python wheel or sdist directories, Maven repositories, bootstrap source archives, source-root provider metadata, and provider proof inputs where represented as source/input material.
- Keep source-bundle export and import independent of build execution and remote-builder authorization.
- Make offline preflight consume the imported source bundle and report missing/stale inputs before sandbox execution.
- Make import atomic and crash-safe, with explicit pin/root behavior so imported source payloads are not garbage-collected before the planned build consumes them.
- Preserve proof-before-claim wording: a valid source bundle proves input availability and identity only, not build success, compiler correctness, or remote output trust.

## Non-goals

- No output closure export/import; that remains `store-archive-transport`.
- No live remote-builder transport; that remains `p2p-remote-builders`.
- No implicit network fetching during import or verify.
- No use of ambient package-manager caches, VCS caches, target/build directories, or checkout caches as undeclared input sources.
- No claim that a source bundle proves reproducibility or build success by itself.

## Target Spec Domains

- `source-transports` for source bundle format, export/import/list/verify, and offline preflight semantics.
- `verification-evidence` may later receive cross-domain proof-before-claim text if source-bundle evidence becomes release-facing.
