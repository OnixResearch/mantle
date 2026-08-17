# Implementation evidence

Date: 2026-08-02

## Exact preserved path route

`preserve-cache-paths-v1` accepts one unchanged source and target prefix.
The graph compiler retains exact output and source paths.
The executable plan binds the separate `cache-only-preserve-v1` route into its BLAKE3 identity.

Plan validation rejects unsupported routes, changed prefixes, changed path maps, missing cache hints, and missing non-claims.
It treats exact identity maps as target facts.
It still rejects valid, unmapped foreign store paths.

## Cache closure preflight

The cache closure planner now limits members, cumulative references, depth, NARInfo bytes, and total NAR bytes.
It canonicalizes closure metadata and binds the plan with BLAKE3.

The HTTP importer now accepts a pure closure validator.
It discovers and verifies complete metadata before any NAR download.
The realization validator permits only exact paths from the executable plan.

Local HTTP tests cover these cases:

- complete dependency-first hydration;
- complete local reuse;
- a missing dependency;
- an untrusted signature;
- a changed returned path;
- a changed NAR;
- member, reference, depth, NARInfo, and total-byte limits;
- duplicate references;
- validator rejection before NAR transfer or PathInfo mutation;
- dependency content failure without root admission.

## Scheduler observation

Cache-only realization requires online substitution, one selected output root, one cache URL, and an empty source bundle.
Trusted public keys come from indexed query fields in the plan's cache URL.
Duplicate indexes and unsupported query fields fail closed.

After hydration, Mantle reopens the store without remote services.
It registers only selected root units.
The cache observer removes all input derivations, input sources, arguments, and executable builder authority.
The ordinary registry, scheduler, report, root retention, and store lookup remain active.

## Receipt and audit facts

The realization receipt binds its route, cache policy BLAKE3, and complete ordered closure facts.
Each fact records path, NAR hash, NAR size, references, signature names, depth, and transfer disposition.
Unneeded graph units use `not-required-cache-only`.

The audit derives cache-only closure membership and references from the self-digest-valid realization receipt.
It verifies admitted PathInfo and scans directory and blob services.
Identity path maps do not become untranslated-reference findings.

The scanner also distinguishes generic gzip streams from CPIO initrds.
It recognizes bounded libtool archive metadata.
Byte-reference admission requires a valid store path digest.

## Configuration and documentation

Typed Nickel contracts own cache closure limits and the preserved Nix translation policy.
The freshness check exports both generated JSON files.
Positive exports and three negative policy fixtures run in `check-nickel-configs`.

The README, trust model, operator guide, machine-artifact guide, and inventory describe the route and its claim boundary.
