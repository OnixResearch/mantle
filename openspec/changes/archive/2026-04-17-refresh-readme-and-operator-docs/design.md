## Context

The current repo already has three layers of operator-facing documentation:

- `README.md` as the first-stop overview
- focused docs such as `docs/benchmark-suite.md`
- proof and claim-boundary material in `docs/bootstrap-stage0-inventory.md`

That structure is good enough, but the content has drifted.

The CLI now ships operator surfaces that a reader cannot reconstruct from the
current README command summary alone: `doctor`, `build --plan`, `attest`,
`release`, `shell`, `develop`, and strict hermetic build selection. At the
same time, the repo already has deeper benchmark and bootstrap docs that must
stay aligned with the same command surface and the same claim boundaries.

## Goals / Non-Goals

**Goals:**
- make the README match the shipped operator workflows
- keep benchmark and bootstrap/release docs aligned with the current tree
- keep release-evidence language bounded and consistent across documents
- improve doc navigation with deliberate cross-links instead of scattered prose

**Non-Goals:**
- adding new CLI features or changing runtime behavior
- turning the README into a flag-by-flag manual
- rewriting every doc page in the repo
- weakening claim boundaries to make release or bootstrap workflows sound
  stronger than current evidence supports

## Decisions

### 1. Keep the README workflow-first

**Choice:** use the README as the entry point for operator workflows, not as a
full reference manual.

**Rationale:** the README is already large. It should show the supported command
families and point readers at the right deeper doc instead of absorbing every
example.

**Implementation:** update README sections for diagnostics/planning,
dev-shells, attestations, release evidence, and strict hermetic builds; add or
refresh links to focused docs where detail belongs.

### 2. Use focused docs for deeper workflow detail

**Choice:** keep benchmark detail in `docs/benchmark-suite.md` and keep
bootstrap/release claim-boundary detail in
`docs/bootstrap-stage0-inventory.md`. Add a new focused doc page only if the
README would otherwise become harder to scan.

If a new focused doc page is created, it must have three things:
- a clear README link into it
- at least one current command example for the workflow it explains
- wording checked against the same main specs as the README summary

**Rationale:** benchmark workflow and bootstrap claim boundaries are already
specialized topics. They read better as stable detail pages than as long README
subsections.

**Implementation:** rewrite those focused docs only where they drift from the
current command surface, current entry points, or current claim language.

### 3. Treat code and main specs as the source of truth

**Choice:** audit doc text against `src/main.rs`, command help output, and the
current main specs instead of preserving legacy prose.

**Rationale:** this is a documentation-alignment change. The cheapest way to
avoid another stale pass is to start from shipped behavior and normative spec
text.

**Implementation:** use an explicit audit pass for command families, flags,
workflow claims, focused-doc links, and proof/release wording before editing.

### 4. Keep claim boundaries identical across README and proof docs

**Choice:** describe `crunch release verify` as bundle-local integrity and
proof-context verification, not as proof of full-source bootstrap or global
reproducibility.

**Rationale:** the repo already distinguishes packaged evidence from broader
bootstrap claims. Divergent wording between README and bootstrap-facing docs
would reintroduce the same confusion this change is trying to remove.

**Implementation:** align the README release-evidence wording with
`docs/bootstrap-stage0-inventory.md` and the release-evidence spec in the same
edit pass, then cross-check the inventory language against the current proof
modes and current reduced seed/provider description in the repo.

## Verification

- compare updated README workflows against `crunch --help`, `crunch build
  --help`, `crunch shell --help`, `crunch attest --help`, and
  `crunch release --help`
- reread the benchmark doc against the performance spec and the checked-in
  benchmark entry points
- reread README and bootstrap-facing docs against the CLI,
  release-evidence, and validation specs, plus the current proof-mode and
  reduced-seed notes in the repo
- run `openspec validate refresh-readme-and-operator-docs`

## Risks / Trade-offs

**[README grows again]** Adding every operator surface inline can make the
README harder to scan. Mitigation: keep summaries short and move detail to
focused docs.

**[Docs still drift later]** One cleanup pass can decay. Mitigation: structure
sections around stable command families and keep the README linked to the few
focused docs that carry the detail.

**[Claim wording gets stronger by accident]** Release and bootstrap wording is
sensitive. Mitigation: verify the final text against the existing bounded claim
language in the main specs before landing.