# Bootstrap blocker inventory report

- Schema version: 1
- Enforcement mode: true
- Findings: 1
- Marker classes present: 1
- Promotion claims: 1

## Marker classes

- `bridge-output`: 1 finding(s) — Bootstrap stage uses or documents a bridge output rather than end-to-end source-built proof.
- `compiler-runtime-crash-boundary`: 0 finding(s) — Known compiler/runtime crash, timeout, or signal boundary still gates promotion evidence.
- `legacy-provider-fallback`: 0 finding(s) — Legacy musl.cc or host-provider fallback remains part of the bootstrap path or documentation.
- `normalization-only-provider`: 0 finding(s) — Provider contract is normalized but not yet accepted as full source-built proof.
- `placeholder-deferred`: 0 finding(s) — Placeholder, TODO, or deferred full-source work remains in a bootstrap-critical surface.
- `prerequisite-gated-evidence`: 0 finding(s) — Evidence is explicitly prerequisite-gated or records a blocked status rather than promotion.

## Promotion claims

- /tmp/tmp.GI367R6oTt/claim.md:1 — `full-source bootstrap status: promoted`

## Findings

- `bridge-output` /tmp/tmp.GI367R6oTt/blocker.ncl:1 — `# bridge output remains, so promotion must fail`
