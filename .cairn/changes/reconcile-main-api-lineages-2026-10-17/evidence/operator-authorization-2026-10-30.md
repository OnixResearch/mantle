# Operator authorization, 2026-10-30

## Source

The session operator replied to an assistant status message with one line, quoted
verbatim (spelling preserved):

> authroize all

The assistant's message asked a question and listed two groups. It first listed
"Only you can unblock" items 1-7: the frozen G1/G2 gate route, the cold source-built
cohort, the Rust toolchain, three ADR acceptances, spec syncs and archives,
retrospective waivers, and the fault-injection change's upstream prerequisite.
It then offered three engineering options under "What I can start without you"
(the ordinary Rust-pipelining baseline harness, the build-authority grant sites,
and the thin-CLI I6/V1/V2 audit), and asked "Which should I take?".

This file records the assistant's reading of the reply as authorizing every listed
item and all three engineering options. It is the assistant's interpretation, not
the operator's text. If the operator intended a narrower scope, correct this file
first; no action recorded below should be read as broader than its row.

## Reading, item by item

| # | Item | Authorized action | Not authorized, and not supplied |
|---|------|-------------------|----------------------------------|
| 1 | Frozen G1/G2 gate route | (a) A specifically reviewed, bounded, read-only observer that completes the owned-target argv/environment equality decision despite protected process environments, through an already-authorized mechanism, with product and monitor execution staying unprivileged. (b) Admission of the finalized isolated tracked-Git subject as the successor source, and an additive reviewed launcher binding that selects its working directory and flake while preserving the original run root, target, temp and stores, baseline, time and credit, caps and stops, shell cohort, pins, profiles and serial one-leg custody. | Running the full monitor under sudo. Ignoring `EACCES`. Limiting the scan to the current UID or to known PIDs. Treating the lock as sufficient. Reusing the task5 ownership result. Changing proc mount options, permissions or capabilities. Editing the sealed manifest or the v3 monitor. Changing caps, baseline, epoch or credit. No credential is supplied: a step that needs root is run by the operator and its output retained. |
| 2 | Cold source-built cohort | Operator admission, as owner, of a source-profile, lineage and provider pin set that the assistant derives from the frozen final source, and running the cold proofs. | This file names no digest, so it admits none. A pin is admitted only when its digest and derivation are recorded in evidence. Any review the proof guide requires beyond the owner remains required. |
| 3 | Rust toolchain | Adopting a toolchain and provider that accept the early-metadata options, and a delegated finite cgroup session, if one can be obtained. | Authorization cannot create a compiler, a source-fixed provider or a cgroup delegation. Only re-probes of available toolchains are possible; delegation needs the operator. |
| 4 | ADR acceptance | Acceptance of ADR 0059, ADR 0060 and ADR 0095 as decisions, after assistant review that the artifacts each names exist. | Acceptance of any open task of the changes that own them. |
| 5 | Spec sync and archive | `cairn sync` and `cairn archive` for a change whose own tasks are all checked and whose gates pass, run through `scripts/cairn-lifecycle-evidence.rs` with captured transcripts, after the implementation commit. | Archiving a change that still has an open task or a failing gate. |
| 6 | Retrospective waivers | Accepting watch T4.1 and demand T4.3 on after-side evidence, with the before side waived. Each waiver states its basis in its own file. | Any claim that a historical before run passed. |
| 7 | Fault-injection upstream prerequisite | Verifying and evidencing the existing upstream pilot commit in an isolated worktree of the independent repository. | Landing it: the upstream `main` checkout has unrelated uncommitted changes overlapping the files the pilot edits, so the fast-forward is left to that repository's owner. Archiving the Mantle change before the landing is recorded. |
| 8 | Engineering options | All three, delegated to isolated agents in separate git worktrees whose branches the integrator validates before merging. | Agents do not tick checkboxes, edit ADR statuses, sync specs or archive. |

## Limits that remain

- An authorization is not evidence. Each task still needs its own captured, exact
  evidence, and a checkbox records only what that evidence supports.
- Model output is not completion authority. Delegated results are validated by the
  integrator before anything is accepted.
- No task is closed, no spec synced and no change archived by this file.
