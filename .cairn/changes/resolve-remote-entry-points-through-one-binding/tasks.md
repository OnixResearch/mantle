# Tasks: Resolve remote entry points through one binding step

Only the individually evidenced contract, implementation and fixture tasks below are checked; baseline, whole-gate and lifecycle acceptance remain open. Evidence is bounded to the retained v8/v13 subject, not current-tree acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record every remote entry point, its name shape, its admission checks, and its current revocation behavior. r[remote_builds.single_binding_resolution]
- [x] [serial] T1.2 Define the resolve step, the binding record, the generation fence, and the revocation operation. r[remote_builds.single_binding_resolution]
- [x] [serial] T1.3 Record the generation-bound, retraction-not-deletion, and fail-closed-early decisions in an ADR. r[remote_builds.binding_revocation]

## Phase 2: Core and shell

- [x] [serial] T2.1 Implement pure name validation, binding selection, generation comparison, revocation state, and resolve decision. r[remote_builds.single_binding_resolution]
- [x] [serial] T2.2 Serve one resolve step per entry point and persist bindings with explicit generation. r[remote_builds.single_binding_resolution]
- [x] [serial] T2.3 Implement revocation as a retraction of one binding and refuse new sessions under it. r[remote_builds.binding_revocation]
- [x] [serial] T2.4 Keep existing ticket, substituter, and base-layer checks behind the resolve step without weakening them. r[remote_builds.single_binding_resolution]

## Phase 3: Fixtures

- [x] [parallel] T3.1 Add positive fixtures: resolve once yields a session handle; resolving the same name twice yields the same generation; each existing entry point resolves through the step. r[remote_builds.single_binding_resolution]
- [x] [parallel] T3.2 Add negative fixtures: expired name, forged name, stale generation, mismatched prefix, unknown binding, and resolve before payload work. r[remote_builds.single_binding_resolution]
- [x] [parallel] T3.3 Add revocation fixtures: new session refused after revocation; already-admitted content still verifies; recreating a name yields a new generation. r[remote_builds.binding_revocation]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the resolve, generation, and revocation rails before and after the change. Preserve exact results. r[remote_builds.binding_revocation]
- [ ] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[remote_builds.single_binding_resolution]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[remote_builds.binding_revocation]

## Retained individual task support (2026-10-26)

`evidence/task-closure-gate-v13-2026-10-26.json` maps T1.2, T1.3, T2.1–T2.4 and T3.1–T3.3 to exact v13 source digests, named passing tests and the pinned v8 endpoint smoke. Each named source still matches the v13 start snapshot. The smoke reports all four endpoint kinds, isolated exact revocation and generation 1 fenced by 2. T1.3 records ADR 0128 as **Proposed**, not approved. The earlier failed cache-binding fixture remains in historical evidence; its named v13 result is now `ok`.

`evidence/focused-tests-gate-v13-2026-10-26.json` lists four passing source-binding tests. The v13 gate's non-flake steps exited 0, but flake exited 1 because its source filter omitted compile-time Python inputs; this is a repository failure, not a host limitation. T1.1 and T4.1–T4.3 remain open: no before-change baseline, full quality-gate acceptance, spec sync or archive is manufactured.
