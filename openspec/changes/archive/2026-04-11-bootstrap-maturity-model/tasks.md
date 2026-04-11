## Phase 1: Inventory the current bootstrap story

- [x] Write a command-by-command trust inventory for `crunch bootstrap`, `crunch bootstrap --fetch`, `crunch self-build`, and `./scripts/prove-self-hosting.sh`.
- [x] Classify the guarantees each path provides today: seed-assisted bootstrap, self-hosting proof, full-source bootstrap, or reproducible release evidence.
- [x] Name the remaining trust anchors and blockers: fetched musl-gcc seed, host Rust/tooling, source-staging helpers, and lack of independent reproducibility evidence.

## Phase 2: Turn Bootstrappable best practices into a repo checklist

- [x] Record the current yes/partial/no status for the Bootstrappable Builds guidance most relevant to crunch: alternative build path, bootstrap-binary provenance labeling, bootstrap-binary reproducibility, and automated traceability.
- [x] Attach concrete repo evidence to each checklist line (`cargo build --release`, pinned `FETCH_SEEDS`, `./scripts/prove-self-hosting.sh`, and README caveats about what is not yet proven).
- [x] Turn every "partial" or "not yet" checklist item into a named follow-up milestone or trust-reduction task.

## Phase 3: Tighten docs and specs

- [x] Update `README.md` (and any dedicated bootstrap doc that gets added) with a bootstrap maturity section that distinguishes current guarantees from future goals and includes the best-practice checklist.
- [x] Update `openspec/specs/bootstrap/spec.md` so the bootstrap spec requires explicit trust inventories, claim boundaries, and the best-practice checklist framing.
- [x] Add a short roadmap section listing the next trust-reduction milestones instead of implying that today’s self-hosting proof is the end of the bootstrap story.

## Phase 4: Verify wording against reality

- [x] Run `openspec validate bootstrap-maturity-model`.
- [x] Re-read the proof helper output and README wording together to confirm the repo does not claim full-source bootstrap or reproducible fixed points without current evidence.
