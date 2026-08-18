## ADDED Requirements

### Requirement: Source bundles realize declared fetcher inputs offline

r[source_transports.source_bundle_realizes_fetcher_inputs] Mantle MUST allow imported and pinned source-bundle records to satisfy declared fixed-output fetcher inputs without live network access. Source-bundle realization MUST verify source-record identity, BLAKE3 payload digest, expected fixed-output hash and mode, source kind, logical store prefix when present, and VCS revision when applicable before admitting the materialized source input into store or castore state.

#### Scenario: imported fixed URL source becomes build input

GIVEN a selected build root depends on a fixed-output URL source
AND local source state contains a pinned imported source record with matching identity, URL metadata, content digest, and expected fixed-output hash
WHEN Mantle realizes inputs for an offline build
THEN Mantle MAY materialize the source input from source state without contacting the network
AND the dependent build MUST consume the verified local source material.

#### Scenario: imported VCS snapshot must match revision

GIVEN a selected build root depends on a fixed-output VCS snapshot
AND local source state contains a pinned checkout payload for the same source identity
WHEN Mantle verifies source-bundle realization
THEN Mantle MUST verify the requested revision or equivalent immutable identity before admitting the input
AND a wrong, missing, or ambiguous revision MUST fail before sandbox execution.

#### Scenario: stale source state fails before execution

GIVEN imported source state is missing, stale, unpinned, unsupported, untrusted, network-required, wrong-prefix, wrong-kind, or fails the fixed-output hash check
WHEN an offline build attempts to realize that input
THEN Mantle MUST reject the source-bundle realization before sandbox execution
AND diagnostics MUST name the source record and blocker class.

#### Scenario: source-bundle realization remains a bounded claim

GIVEN source-bundle realization admits a source input
WHEN Mantle reports that result
THEN the report MAY claim only that declared source/input material was identity-matched and locally available
AND it MUST NOT claim final build output correctness, output trust, compiler correctness, or reproducibility without separate evidence.
