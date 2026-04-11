## Why

The references we just reviewed make one thing clear: serious bootstrap work
needs a precise story about trust roots, stage boundaries, and what is or is
not proven today.

- Bootstrappable Builds focuses on minimizing opaque bootstrap binaries and
  making the remaining seeds explicit.
- Guix's full-source bootstrap write-up names the exact trust root
  (357-byte `hex0-seed`), the staged package graph above it, and the
  remaining bootstrap driver that still needs reduction.
- StageX draws a hard line between distinct properties: full-source
  bootstrapping, reproducibility, and release-signing policy.

crunch already has real progress here: `bootstrap --fetch`, a from-source
bootstrap chain, and a checked-in self-hosting proof. But the repo still does
not present one explicit maturity model for bootstrap claims. A contributor can
see that stage1 can rebuild stage2, but they cannot quickly answer:

- what binaries or tarballs are still trusted seeds,
- which host tools are still required,
- whether crunch currently claims self-hosting, full-source bootstrap,
  reproducible release builds, or something narrower,
- what the next trust-reduction milestones are.

That leaves too much room for overclaiming, underspecifying, or mixing
separate guarantees together.

## What Changes

- Add a bootstrap maturity model that distinguishes seed-assisted bootstrap,
  self-hosting proof, full-source bootstrap, and reproducible release claims.
- Require an explicit trust-inventory for each supported bootstrap path:
  `crunch bootstrap`, `crunch bootstrap --fetch`, `crunch self-build`, and the
  checked-in self-hosting proof.
- Add a concrete checklist against Bootstrappable Builds best practices,
  especially the guidance for build-system writers and distributions.
- Add a staged roadmap for reducing remaining trust anchors and host-tool
  dependencies.
- Tighten the bootstrap spec and docs so they say exactly what crunch proves
  today and what remains future work.

## Capabilities

### New Capabilities
- `bootstrap-trust-inventory`: contributors can see current trusted seeds,
  fetched artifacts, and host prerequisites per bootstrap path
- `bootstrap-maturity-classification`: docs and specs classify bootstrap claims
  instead of using one overloaded “self-hosting” label
- `bootstrap-roadmap`: remaining trust-reduction milestones are tracked in one
  place

### Modified Capabilities
- `bootstrap-documentation`: becomes explicit about evidence, trust anchors,
  and non-goals
- `self-hosting-proof`: documented as one maturity step, not the entire
  bootstrap story

## Impact

- **Files**: `README.md`, bootstrap-focused docs (new or existing),
  `openspec/specs/bootstrap/spec.md`
- **APIs**: none required; this change is documentation + specification first
- **Dependencies**: none
- **Testing**: `openspec validate` for the change, plus doc review against the
  current proof/helper behavior so the written claims match the code and the
  best-practice checklist stays evidence-backed
