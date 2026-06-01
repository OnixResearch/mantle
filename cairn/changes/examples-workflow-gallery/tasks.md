# Tasks

## Gallery expansion

- [ ] [serial] Reorganize `examples/README.md` into progressive lanes: beginner, fetcher cookbook, package composition, project workflow, trust/provenance, and advanced bootstrap. r[examples.progressive_gallery]
- [ ] [serial] Add at least one small local example that demonstrates consuming another package or a named output layout, then wire it into catalog and validation. r[examples.progressive_gallery]
- [ ] [serial] Expand `examples/project/` docs or fixtures so default package, named package, and checks have explicit commands and expected outputs. r[examples.progressive_gallery]

## Trust/provenance lane

- [ ] [serial] Add a lightweight runnable provenance or attestation example that produces deterministic local evidence, or record an oracle checkpoint explaining why only command recipes are safe for this slice. r[examples.trust_provenance_gallery]
- [ ] [serial] Add smoke or shape tests for any runnable provenance example, including positive evidence-shape assertions and negative non-claim wording checks for skeleton proof workflows. r[examples.trust_provenance_gallery]
- [ ] [serial] Ensure examples remain frontend-neutral and do not reintroduce Onix/NixOS-style module-layer semantics under `examples/`. r[examples.progressive_gallery]

## Verification

- [ ] [serial] Run focused examples gallery/catalog tests and record output. r[examples.progressive_gallery] r[examples.trust_provenance_gallery]
- [ ] [serial] Run existing module-boundary guard tests that scan `examples/` and record output. r[examples.progressive_gallery]
- [ ] [serial] Run `cairn validate --root .` and the tasks gate, then archive only after completed tasks cite durable evidence. r[examples.progressive_gallery] r[examples.trust_provenance_gallery]
