# Radiance reference portfolio search

## Frame

Goal: produce an optional, offline, two-route Radiance bootstrap fixture with exact source, predecessor, execution, convergence, publication, and non-claim evidence.

Completion requires:

- three admitted Git SHA-256 source observations;
- one pinned source bundle with no proof-time network use;
- a measured host C root, built emulator, and built C99 bootstrap compiler;
- seed and C99 routes that each execute every immediate predecessor;
- route-local fixed points and a separate cross-route comparison;
- a replayable receipt and immutable published artifacts;
- positive and negative validation.

False completion includes one route only, mutable refs, copied source, live proof fetches, ambient compiler discovery, skipped predecessors, unchecked process success, or correctness claims from equality.

Audit risks include Git SHA-256 support, seed/source drift, C99 language drift, module limits, emulator compatibility, child-process authority, source-path leakage, and upstream fixed-point overclaims.

Budget: three external repositories, three compatibility probes, one surviving route pair, no subagents, and one live proof after deterministic fixtures pass.
Allowed outcomes are validated or an exact bounded blocker.

## Serial lens A: use the proposed revisions unchanged

- Family: proposed-current-source.
- Mechanism: Radiance `673ae4f6...`, radiance.s0 `7834d3a9...`, emulator `92cdb0c5...`.
- Claim: both routes can compile the proposed Radiance source.
- Evidence: the seed route reached S2/S3 equality.
- Failure: radiance.s0 stopped before stage one with `maximum number of modules (48) exceeded`.
- Further check: an older 42-module Radiance source passed the module count but failed on unsupported syntax.
- State: falsified.

The proposed Radiance revision is not a Git tag. It is an immutable SHA-256 commit reachable from a remote branch.
Its checked-in `seed/radiance.rv64.git` value names `d024ed30...`, which is not present in the published repository object graph.

## Serial lens B: patch the C99 bootstrap compiler

- Family: source-transform.
- Mechanism: increase limits and backport syntax support into radiance.s0.
- Claim: a modified C99 route could compile current Radiance.
- Gap strength: stronger than the requested exact-source cohort.
- Blocker: the required patch would change admitted third-party source and add a new compiler-maintenance surface.
- State: rejected.

A limit-only change is insufficient because the current source also uses syntax that radiance.s0 does not parse.

## Serial lens C: choose the exact compatible source cohort

- Family: frozen-compatible-cohort.
- Mechanism: Radiance `0d8a2d4f...`, unchanged radiance.s0 `7834d3a9...`, unchanged emulator `92cdb0c5...`.
- Claim: two exact unmodified routes can reach the same fixed point.
- Evidence:
  - seed route: checked seed and S1 were identical;
  - C99 route: S2 and S3 were identical;
  - route outputs matched byte-for-byte;
  - output length was 1,285,492 bytes;
  - output BLAKE3 was `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`;
  - upstream SHA-256 interoperability value was `d4a99e751e28a96c140ee078a18e74afbb95a4ee5f50158fd777d3d4fdab7612`.
- State: validated for unprotected research execution.

All three repositories report Git object format `sha256` and carry the same MIT license bytes.
The license BLAKE3 is `d0f870e9de0345cb08c6ecc3367d99c12a89756d23cfec7f8260aaf1dcc888b6`.

## Surviving design

Use the frozen compatible cohort from lens C.
Treat every revision as an explicit-format immutable commit, not a Git tag.

Use `crunch-source-core` for observations and source-bundle monotonic ingest.
Use a new narrow no-std core for cohort, graph, convergence, and receipt policy.
Use the existing ptrace protected-execution shell for exact native executable decisions.
Use the pinned durable publication component for selected immutable artifacts.

The live proof must reproduce the research result from an imported bundle.
It must not import upstream claims that fixed-point equality proves compiler correctness, determinism, or seed trust.

## Oracle checkpoint

- Question: Can unchanged Radiance sources produce an optional, protected,
  offline two-route convergence receipt without extending Mantle's bootstrap
  claims?
- Inspected evidence: three Git SHA-256 repositories, their exact source and
  license identities, incompatible-revision probes, the research route pair,
  V14 protected execution, and the V15 eight-role receipt.
- Decision: Accept the frozen cohort and the per-translation-unit compiler
  roots with a separate single-threaded linker root. Reject source patches,
  mutable refs, live fetches, and correctness attribution.
- Owner: Mantle owns source admission, protected execution, receipt policy,
  publication, and non-claims. The Radiance projects retain their source and
  compiler meaning.
- Next action: preserve V15 replay evidence, run focused quality and Cairn
  gates, sync the accepted requirement, and archive the change.
