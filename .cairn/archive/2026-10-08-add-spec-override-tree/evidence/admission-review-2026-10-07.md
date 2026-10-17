# Admission review: package-layer design priors (2026-10-07/08)

This review covers both `add-spec-override-tree` and
`adopt-data-only-dependency-exports`. Both proposals defer to one shared
package-layer admission decision, so this file appears unchanged in both
changes. The decision is recorded in ADR 0109.

## Rule under review

The workspace rule (`AGENTS.md`, "Shared conventions") says: "Foundation work
must name a current consumer, target outcome, adoption path, and maintenance
owner. Reject infrastructure without concrete demand."

Each change's original gate (preserved in `evidence/original-scope/`) asked
for the same four facts:

- `mantle.spec_override_tree.bounded_adoption_gate` asks for "the first
  consumer, shared with or explicitly aligned to the package-layer admission".
- `mantle.dependency_exports.bounded_adoption_gate` asks for "the first
  consumer, its target outcome, the adoption path, and the maintenance owner".

Each original proposal states, under Impact: "Immediate consumer: none yet".

## Revisions searched

- **Mantle:** published main `e24bbbc2f803f59872e2e59370bd8ce0829c918e`. The
  local maintainer checkout `da00f584`, which includes uncommitted work, was
  read for unpublished candidates.
- **Sibling repositories** (`/home/brittonr/git/OnixResearch/<repo>` HEADs on
  2026-10-07):

| Repo | HEAD |
|---|---|
| onixos | 7b2b35efbc7b |
| onixpkgs | 85623f16b4c8 |
| kiln | e1e09fded0c5 |
| aspen | 81baf80cb667 |
| kamacite | 980743455f53 |
| lattice | b6e318d6dc23 |
| site | 012905a97574 |
| trellis | d91188e495d6 |
| tile | 5e703953840b |
| chaoscontrol | 06ac3cdb05c2 |
| cairn | 007ba5193dd8 |
| octet | 4a540dff99aa |
| nickel-export | 07ea21c0666f |
| animus | ce968741e276 |
| basalt | f79662d3528c |
| valence | 539d756494c3 |
| artifact | 2a34ddd92a4e |

## Searches and results

Every command ran read-only from the workspace root.

1. `rg -l --glob '*.ncl' -e mk_derivation -e builders/lib -e overrideAttrs -e spec_override -e override_tree -e dependency_exports -e exports_record <17 sibling repos>`
   - Result: one file, `onixos/lib/exports.ncl:79-82`.
   - That file is a helper over OnixOS service-module exports (upstream
     module, instance, role and machine maps). It is unrelated to Mantle
     package specs or dependency outputs, so it is not a consumer.
2. `rg -l -e spec-override-tree -e spec_override_tree -e data-only-dependency-exports -e dependency_exports <17 sibling repos>`
   - Result: no matches.
3. Mantle, in-repo consumers of the builders layer: `rg -l 'builders/|mk_derivation' --glob '*.ncl' --glob '*.rs'`
   - Matches: `builders/`, `examples/` (including `examples/package-set.ncl`),
     tests (`tests/integration.rs:2141-2251`, `tests/release_cli.rs`,
     `tests/remote_*`, `tests/examples_workflow_gallery.rs`), and branding
     and archive scripts.
   - There is no production caller.
4. Mantle bootstrap recipes: 185 `bootstrap/*.ncl` files.
   `rg -l 'import "(\.\./)?builders' bootstrap` matches 0 of them.
5. Mantle overrides: `overrideAttrs` is defined at
   `builders/mk_derivation.ncl:262`. Its only callers are
   `examples/override.ncl:28` and the catalog entry
   `examples/catalog.ncl:276` ("Demonstrates overrideAttrs without rewriting
   the original package").
6. Mantle dependency behavior: `builders/mk_derivation.ncl:39-45` documents
   `buildInputs`/`nativeBuildInputs` as "Their /bin dirs are added to PATH".
   No setup hooks exist for an exports record to replace.
7. mantlepkgs variation uses identity-bound records: `VariantContract`
   (`mantlepkgs/domains/contracts.ncl:60`), `variants | Array VariantContract`
   (`:107`), exported as `Variant` (`:115`).
8. No Mantle package-layer family exists. No active or archived change
   defines one; only these two changes mention a "package layer".
9. OnixOS layering is its own closed `Foo` + `FooOverride` pattern
   (`onixos/lib/onix.ncl:18-19`).
   - Kiln consumes only the raw `mantle.Derivation` contract
     (`kiln/config/fixtures/mantle-build-plan.ncl:1-6`).
   - onixpkgs is a Nix overlay, not Mantle specs.
10. Unpublished candidate:
    - `run-cargo-build-scripts-as-plan-units` exists only as untracked files
      in the maintainer checkout and is absent from published main. It has
      0/16 tasks done and depends on the unaccepted
      `build-cargo-units-as-dynamic-plans`.
    - It calls itself "a candidate first consumer for
      `adopt-data-only-dependency-exports`, subject to that change's admission
      gate" (its `proposal.md:57-60`).
    - Its own contract declares native inputs by role name with
      `{{role:<name>}}` markers and pkg-config directories, and already forbids
      dependency hooks. It needs neither tree-derived exports records nor a
      typed build-system registry. [INFERENCE from its design.md, not from
      running it.]
    - A change that is not started and not published is not a current
      consumer.

## Decision

| Fact required by the gate | Found |
|---|---|
| Current consumer | none |
| Target outcome | none beyond the design prior itself |
| Adoption path | none: no package layer; the bootstrap recipes and mantlepkgs use other mechanisms |
| Maintenance owner | "assigned at admission" in both proposals; nobody is assigned |

Both changes are **rejected at admission** and closed as decision-only
`no-spec-delta` changes, as ADR 0109 records. No implementation task was
started or performed.

## Non-claims

- This search covers the listed repositories and revisions only.
- A text search can miss consumers that use other names. The revisit
  triggers in ADR 0109 cover later demand.
- The rejection makes no judgment about the quality of the `repkgs`-derived
  designs.
