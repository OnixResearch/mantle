# Tasks: Resolve remote entry points through one binding step

All tasks remain open. Creating this proposal is not producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record every remote entry point, its name shape, its admission checks, and its current revocation behavior. r[remote_builds.single_binding_resolution]
- [ ] [serial] T1.2 Define the resolve step, the binding record, the generation fence, and the revocation operation. r[remote_builds.single_binding_resolution]
- [ ] [serial] T1.3 Record the generation-bound, retraction-not-deletion, and fail-closed-early decisions in an ADR. r[remote_builds.binding_revocation]

## Phase 2: Core and shell

- [ ] [serial] T2.1 Implement pure name validation, binding selection, generation comparison, revocation state, and resolve decision. r[remote_builds.single_binding_resolution]
- [ ] [serial] T2.2 Serve one resolve step per entry point and persist bindings with explicit generation. r[remote_builds.single_binding_resolution]
- [ ] [serial] T2.3 Implement revocation as a retraction of one binding and refuse new sessions under it. r[remote_builds.binding_revocation]
- [ ] [serial] T2.4 Keep existing ticket, substituter, and base-layer checks behind the resolve step without weakening them. r[remote_builds.single_binding_resolution]

## Phase 3: Fixtures

- [ ] [parallel] T3.1 Add positive fixtures: resolve once yields a session handle; resolving the same name twice yields the same generation; each existing entry point resolves through the step. r[remote_builds.single_binding_resolution]
- [ ] [parallel] T3.2 Add negative fixtures: expired name, forged name, stale generation, mismatched prefix, unknown binding, and resolve before payload work. r[remote_builds.single_binding_resolution]
- [ ] [parallel] T3.3 Add revocation fixtures: new session refused after revocation; already-admitted content still verifies; recreating a name yields a new generation. r[remote_builds.binding_revocation]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the resolve, generation, and revocation rails before and after the change. Preserve exact results. r[remote_builds.binding_revocation]
- [ ] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[remote_builds.single_binding_resolution]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[remote_builds.binding_revocation]
