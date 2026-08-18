# Review gap closure

Task-ID: review-gap-closure
Covers: verification_evidence.tracey_coverage_readiness

## Question

How should the review warnings about unsupported final hygiene claims and repeated prompt-omission guidance be handled without overstating the Tracey backfill state?

## Inspected evidence

### Durable post-commit hygiene for prior evidence commit

After commit `aba68bdc record Tracey hygiene checkpoint` and before creating this follow-up evidence file, the repo state was inspected:

```sh
git status --short --branch
git diff --check
```

Output:

```text
## main...origin/main [ahead 4]
```

`git diff --check` produced no output and exited `0`.

This proves the tracked worktree was clean after `aba68bdc` and that whitespace checks passed before this follow-up evidence file was created. It does not claim future commits are clean unless they have their own current status output.

### Repeated omission promotion hint

`review_metrics action=promotions min=3 last=50` reported:

```text
count=3 [class=omission] [scope=review] [route=prompt]
suggestion: Repeated soft finding: tighten prompt guidance now, then promote into a spec rule or deterministic check if repeats continue.
task: Tighten reviewer/prompt guidance for repeated omission findings, then promote to a spec rule or deterministic check if it persists. Example trigger: Clean tree claim is not backed by supplied durable evidence
sources: done-review=3
```

The specific repeated trigger is final/status claims that are not backed by durable evidence in reviewed files or exact tool output.

## Decision

- Keep the durable post-commit hygiene evidence scoped to the prior commit (`aba68bdc`) and avoid making a new final clean-tree claim unless the final answer includes fresh tool output.
- Tighten repo-local prompt guidance in `AGENTS.md`: final clean/dirty state claims now require either exact live status output in the same response or a committed evidence transcript that records the post-change status.
- Defer a deterministic spec rule/check for this prompt-omission class until it repeats after the guidance update, because the current active change is Tracey coverage readiness and no repo-owned reviewer extension/checker lives in this repository.

## Owner

Mantle maintainer / current agent for `tracey-coverage-readiness-backfill`.

## Next action

If a future review repeats the same clean-tree evidence finding, promote it into a deterministic Cairn task or repo-local checker that rejects final evidence summaries lacking either committed status transcripts or exact same-turn status output.
