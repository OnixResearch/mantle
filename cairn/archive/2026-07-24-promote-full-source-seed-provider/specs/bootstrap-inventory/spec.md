## ADDED Requirements

### Requirement: Source-built bootstrap seed provider promotion

r[bootstrap_inventory.source_built_seed_provider] Mantle MUST select its source-built bootstrap seed provider only after the complete declared compiler, libc, binutils, and normalized-provider chain has produced real runtime-validated artifacts without legacy-provider, pass1-delegation, generated-stub, or undeclared host-tool fallback.

#### Scenario: source-built chain produces the normalized provider

GIVEN the declared bootstrap source closure starts from the audited bootstrap seed and contains the TinyCC, musl, GCC, binutils, and required build-tool sources
WHEN Mantle builds the source-provider candidate
THEN every compiler ladder stage MUST compile its admitted artifacts from declared source inputs rather than substitute empty objects, generated stubs, or wrappers that delegate compiler behavior to a predecessor
AND the normalized output MUST contain executable target-prefixed GCC/G++, assembler, linker, archive, inspection, and strip tools plus headers, CRT objects, `libc.so`, `libc.a`, `libgcc_s.so.1`, and provider metadata bound to the source and output identities.

#### Scenario: provider admission executes positive tool and runtime smoke

GIVEN a candidate normalized provider has been built
WHEN Mantle evaluates it for selection
THEN admission MUST execute bounded C, C++, assembly/link, archive, object-inspection, static-libc, dynamic-libc, and libgcc runtime smoke against the candidate's own compiler internals and sysroot
AND executable-bit checks, `--version` output, contract text, or metadata shape alone MUST NOT satisfy admission.

#### Scenario: incomplete or bridged provider fails closed

GIVEN any reachable candidate stage contains the legacy musl.cc provider, `seed-legacy.ncl`, TinyCC-delegating GCC wrappers, fabricated generator/backend objects, missing compiler internals, missing runtime surfaces, undeclared host tools, or stale/tampered evidence
WHEN construction, admission, selection, or self-build runs
THEN Mantle MUST fail with a deterministic diagnostic before changing the selected seed or emitting source-built-provider success
AND it MUST retain the honest legacy development path or blocked status without relabeling host-assisted source-root materialization as full-source proof.

#### Scenario: selected provider drives authenticated fixed-point self-build

GIVEN the source-built provider passes runtime admission from committed implementation source and its complete fixed-fetch closure is independently authenticated
WHEN a fresh clone runs stage0 → stage1 → stage2 with undeclared live source acquisition forbidden
THEN both stages MUST bind the same source authority and admitted provider identity, report zero undeclared live fetches, and produce matching stage1/stage2 Mantle binary BLAKE3 digests
AND the proof bundle MUST retain provider construction, admission, source-policy, tool/runtime smoke, fallback, and fixed-point evidence.

#### Scenario: source-built-provider claims remain bounded

GIVEN provider construction and fixed-point self-build succeed
WHEN operators or release tooling cite the result
THEN the claim MUST identify the bootstrap seed, source closure, orchestration boundary, platform, provider/output identities, tool/runtime smoke, and fixed-point digests
AND it MUST NOT claim compiler correctness, bootstrap-seed correctness, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, or full Cargo compatibility without separate evidence.
