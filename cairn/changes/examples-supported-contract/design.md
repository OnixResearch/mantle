## Context

Examples are a public onboarding and compatibility surface. Today the support level is encoded in comments, ad hoc tests, and operator memory. Some examples depend on generated `seed.ncl`, some require Linux/bwrap, some hit external networks, and benchmark examples are Cargo examples because root `Cargo.toml` disables `autoexamples`.

## Approach

1. Introduce `examples/catalog.ncl` as the reviewed source of truth. Each entry should include a stable id, relative path, kind, support tier, whether it needs network, whether it needs generated seed material, required tools, expected command family, and expected output shape when applicable.
2. Keep catalog validation pure: parse/evaluate catalog content and checked-in README/index text into in-memory data, then compare entries and paths without touching the network or building derivations.
3. Add tests that reject missing catalog entries, stale README links, unsupported tier strings, and missing expected validation rails.
4. Keep generated seed examples listed as generated/seed-dependent, not as ordinary always-runnable examples.
5. Treat Mantle prose as the user-facing norm. Preserve exact `crunch` command names only where the current binary/package compatibility surface still requires them.

## Risks

- A catalog can become another stale document if tests do not consume it.
- Over-promising support tiers can make CI flaky; real-network and heavyweight examples must stay separate from fast rails.
- Renaming compatibility identifiers prematurely would break existing commands.

## Validation

- Positive tests accept a complete catalog and README pair.
- Negative tests reject missing examples, missing README links, invalid tiers, stale paths, and unsupported Crunch branding in user-facing prose.
- `cairn validate --root .` remains valid after the change package is updated.
