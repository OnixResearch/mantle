## Design

The Nix evidence core is a pure Rust crate/module beneath Mantle release evidence. It validates already-loaded store path refs, derivation/output identity rows, artifact digest metadata, realization labels, and non-claim boundaries. It does not evaluate Nickel, evaluate Nix, run builds, contact substituters, inspect the live store, or start sandboxes.

### Decisions

1. **Store identity is structural.** Store path refs, logical prefixes, derivation names, output names, and output artifact refs are parsed into typed DTOs with deterministic diagnostics.

2. **Derivation/output rows are role-bound.** Build outputs, fixed-output fetches, release bundle members, sidecars, and external evidence rows remain distinct roles.

3. **Caveats are explicit.** Build environment caveats, sandbox caveats, fixed-output caveats, and impure/compatibility caveats must be represented as data before an evidence row is accepted.

4. **Adapters preserve public reports.** Existing `crunch-build-report-v1`, release provenance, and bundle evidence fields retain public names while shared Nix evidence checks move into the core.

5. **No build-correctness promotion.** Passing Nix evidence core validation proves store-shaped identity and bounded metadata consistency only.

### Validation shape

Positive tests cover valid Mantle build report rows, release bundle member rows, sidecar rows, and optional external Nix evidence rows. Negative tests cover malformed store paths, wrong output names, digest mismatch, unsupported derivation identity, missing caveats, ambiguous role labels, missing non-claims, and claims that identity validation proves build correctness or source-code correctness.
