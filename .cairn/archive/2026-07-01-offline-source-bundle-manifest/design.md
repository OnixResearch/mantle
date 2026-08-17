# Design: Offline source bundle manifest

## Architecture

The source bundle is a source/input transport, not an output cache. It sits before evaluation/build dispatch and reuses Mantle's existing source closure, fetcher, package-manager mirror, bootstrap/provider, toolchain/source-root, and BLAKE3 tree hashing primitives.

Keep the functional core pure:

- Pure core: manifest validation, source-record normalization, deterministic ordering, import action planning, missing/stale input classification, source-kind support decisions, compatibility/non-claim classification, and list metadata rendering data.
- Imperative shell: filesystem reads/writes, source tree walking, archive payload streaming, VCS snapshot materialization when explicitly requested by export, provider-state reads, store/source-state persistence, CLI argument parsing, and stdout/stderr.

The shell may fetch or materialize sources only during an explicit export/plan mode that documents network behavior. Import, list, verify, and offline preflight must not fetch from the network.

## Adapter contract

Source bundles are language-neutral. A package-manager or build-ecosystem adapter must translate ecosystem-specific inputs into generic source records plus bounded adapter metadata. Each adapter should declare lock identity, package/source coordinates, expected content refs, offline or network-disable controls, allowed source roots, allowed generated-source directories, cache isolation expectations, and unsupported behavior classes. Cargo is only one adapter fixture; npm/pnpm, Go modules, Python packages, Maven-style repositories, C/C++ source archives, and provider/bootstrap inputs should use the same generic source-record machinery when modeled.

## Manifest shape

Use a Mantle-owned format name such as `mantle-source-bundle-v1`. The manifest should include:

- format version and mandatory/optional feature flags;
- producer identity and created-by tool metadata;
- deterministic root labels and selected build roots;
- bounded source records with kind, logical identity, content ref, byte count when known, source tree digest, executable/mode metadata when modeled, and declared downstream use;
- source-kind-specific metadata for fixed URLs, unpacked archives, VCS checkout snapshots, local path sources, generic language package-manager mirrors with adapter metadata for Cargo/npm/pnpm/Go/Python/Maven-style inputs when modeled, bootstrap source archives, provider manifests, toolchain/source-root inputs, and proof input blobs;
- store-prefix identity only for records that refer to logical store paths;
- optional signature/provenance refs when source material has an existing signed provenance sidecar;
- a BLAKE3 manifest digest and per-payload BLAKE3 digests.

Every list, string, record count, metadata block, and payload chunk must have named limit constants.

## Filesystem canonicalization

Source tree payloads need a reviewed filesystem contract before they can be portable evidence. The bundle should normalize or reject path traversal, absolute payload paths, unsupported file kinds, unsafe symlink targets, device nodes, FIFOs, sockets, ambiguous hardlinks, platform-specific case collisions, invalid UTF-8 where the target contract requires UTF-8, unstable timestamps, and mode bits outside the modeled executable/readable subset. Rejected payloads must fail closed before import or offline preflight readiness.

## Export and plan flow

`mantle source bundle plan` computes a deterministic source/input closure for selected build roots. It reports required sources, already-local sources, unresolved material, network fetches that export would perform, and unsupported source kinds without mutating state.

`mantle source bundle export` writes the plan's supported source records and payloads in deterministic order. Missing required material fails closed unless an explicit narrower mode records the skipped source identities and refuses broad offline-build claims.

## Import and verify flow

`mantle source bundle import` validates the manifest and payload digests, rejects unsupported mandatory features, stores source records under Mantle-owned source/input state, and skips already-present matching records idempotently. It must not accept a source only because the bundle names it; the bytes, digest, kind metadata, and declared identity must match. Import should stage records under temporary identities, verify all digests, then commit atomically; interrupted imports must leave either the old state or a resumable/quarantined partial state that cannot satisfy readiness. Imported records should be pinned or rooted when a planned build depends on them, and unpinned imported material must be reported as GC-eligible rather than durable.

`mantle source bundle verify` checks a bundle or imported source state against a planned build root. It reports `ready`, `missing`, `stale`, `unsupported`, or `untrusted-provenance` classes without executing a build.

## Offline preflight

Offline build preflight consumes planned build roots plus imported source-bundle state. It fails before sandbox execution when any required source/input record is missing, stale, unsupported, or would require network access. The preflight report should be usable by remote-builder input sync and by local offline builds.

## Validation strategy

- Pure positive tests for deterministic manifest digests, record ordering, adapter metadata validation, filesystem canonicalization, import skip/import planning, atomic commit planning, pin/root planning, and offline-ready classification.
- Pure negative tests for unsupported versions, duplicate records, oversized lists/chunks, digest mismatch, source-kind mismatch, stale planned identity, missing required sources, unproven compatibility claims, unsafe filesystem payloads, adapter contract violations, and interrupted import states.
- Store/source-state tests for export/import/list/verify round trips using generated fetcher, generic local path, VCS snapshot, at least one non-Cargo package-manager mirror fixture, Cargo as one adapter fixture, and bootstrap-source fixtures.
- CLI tests proving plan is no-mutate, import/list/verify do not fetch network, and offline preflight fails closed before build execution.
