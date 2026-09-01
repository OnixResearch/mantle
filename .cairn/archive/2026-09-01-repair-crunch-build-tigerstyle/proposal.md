## Why

The strict Tiger Style gate now reaches the build layer. It reports 19
`crunch-build` findings and one `crunch-rustc-wrapper` finding across six source
files. The findings cover unchecked arithmetic, panic paths, recursion,
assertion density, ambiguous interfaces, compound conditions, and quantity
names.

Mantle must repair this debt without lint allowances, scope reductions, warning
baselines, or build-semantic changes. The repair must preserve derivation
identity, content-addressed planning, execution-profile admission, scheduler
failure propagation, rustc-wrapper publication, and public compatibility.

## What Changes

- Replace unchecked bound arithmetic with checked construction.
- Keep malformed build data on typed error paths instead of panic or `expect`.
- Replace recursive structured-attribute canonicalization with bounded
  non-recursive processing over size-admitted JSON.
- Add meaningful assertions only for facts established by prior validation or
  successful effects.
- Replace ambiguous private parameter groups with named input records while
  retaining compatible public call paths.
- Decompose execution-profile conditions and use predicate names without
  changing accepted or rejected profiles.
- Run focused and repository Tiger Style checks with no allowances.

## Impact

- **Files:** the six files named by the initial baseline, two wrapper CLI files
  exposed after the library accepted, focused tests, and lifecycle evidence.
- **Testing:** pre-change and post-change package tests, focused Tiger checks,
  repository Tiger checks, strict Clippy, formatting, full Nix checks, Cairn
  validation, Tracey coverage, and lifecycle gates.
