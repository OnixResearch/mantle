## Context

`mantle nix-free-demo validate` and `mantle nix-free-demo readme` expose validation and rendering for existing summaries. The next operator-facing step is a generator that assembles the expected directory shape from explicit evidence inputs.

The generator should follow functional-core / imperative-shell decomposition: pure manifest construction over input metadata, with filesystem writes in the CLI shell.

## Decisions

### 1. Generator is evidence-assembly, not proof execution

**Choice:** The generator accepts existing proof transcripts, receipt paths, digest values, and status fields. It does not launch expensive proofs by default.

**Rationale:** Demo packaging should be repeatable and cheap; proof execution remains a separate explicit command.

### 2. Validation is reused after generation

**Choice:** After writing a bundle, the command runs the same summary validation path used by `validate`.

**Rationale:** Generated bundles and hand-written bundles should meet one contract.

### 3. Non-claims are required fields

**Choice:** The manifest builder requires explicit status and non-claim text when proof evidence is blocked, synthetic, partial, or demo-only.

**Rationale:** Demo bundles are easy to misread as proof success; the artifact must prevent overclaiming.

### 4. Output is deterministic

**Choice:** Generated JSON ordering, README sections, digest files, and transcript names are stable across equivalent inputs.

**Rationale:** Operators should be able to diff regenerated bundles.

## Risks / Trade-offs

- Generator options can become too broad; keep the first slice minimal.
- If proof status is supplied by hand, validation must catch missing or contradictory fields.
- Large transcripts may need digest-only inclusion with bounded excerpts.
