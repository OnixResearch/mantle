# Tasks: Tighten self-build proof hermeticity

## Phase 1: Strict mode plumbing

- [ ] Thread hermeticity mode into `crunch self-build`
- [ ] Thread hermeticity mode into the checked-in proof helper
- [ ] Record exact bootstrap-tool provenance in proof summaries

## Phase 2: Later-stage enforcement

- [ ] Reject host fallback sandbox-tool discovery in strict later proof stages once crunch-built tool roots exist
- [ ] Keep practical mode reporting fallback use explicitly
- [ ] Add regression coverage for stage1/stage2 fallback rejection

## Phase 3: Validation

- [ ] Re-read the touched bootstrap spec against the final proof behavior before implementation starts
- [ ] Run `openspec validate tighten-self-build-proof-hermeticity`
