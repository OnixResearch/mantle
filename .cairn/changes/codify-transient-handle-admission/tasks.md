# Tasks: Codify transient-handle admission

All tasks remain open. Creating this proposal is not producer acceptance.

## Phase 1: Boundary inventory

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Inventory each protocol boundary: transfer chunks, attempts, checkpoints and acknowledgements, remote sessions, and output admission. r[remote_builds.transient_handle_introduction]
- [ ] [serial] T1.2 For each boundary, record the declaration that introduces the handle or identify the boundary as unenforced. r[remote_builds.transient_handle_introduction]
- [ ] [serial] T1.3 Record the no-feedback and statement-plus-fixtures decisions in an ADR. r[remote_builds.transient_handle_rejection]

## Phase 2: Enforcement and fixtures

- [ ] [serial] T2.1 Add the introduction check where a boundary lacks it, without changing the wire format. r[remote_builds.transient_handle_introduction]
- [ ] [parallel] T2.2 Add one negative fixture per boundary: unknown handle, unknown session, attempt without lease, checkpoint used as content, acknowledgement used as authority, and admission without signed PathInfo. r[remote_builds.transient_handle_rejection]
- [ ] [parallel] T2.3 Add one positive fixture per boundary proving an established handle is accepted. r[remote_builds.transient_handle_introduction]

## Phase 3: Documentation

- [ ] [serial] T3.1 Add the rule and the boundary table to `docs/remote-transfer.md`. r[remote_builds.transient_handle_introduction]
- [ ] [serial] T3.2 Cross-reference the rule from the remote build and substitution documentation where a handle is consumed. r[remote_builds.transient_handle_rejection]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the per-boundary positive and negative rails before and after the change. Preserve exact results. r[remote_builds.transient_handle_rejection]
- [ ] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[remote_builds.transient_handle_introduction]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[remote_builds.transient_handle_rejection]
