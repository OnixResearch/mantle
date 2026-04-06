## Phase 1: Update contradicted specs

- [x] nickel-eval/spec.md: replace "MUST NOT use JSON" with documented JSON round-trip requirement and rationale ✅ 8m (started: 2026-04-05T10:02Z → completed: 2026-04-05T10:03Z)
- [x] nickel-stdlib/spec.md: document open contract as temporary, cross-reference extract-stdlib-builders ✅ 1m (started: 2026-04-05T10:03Z → completed: 2026-04-05T10:03Z)
- [x] ca-derivations/spec.md: replace hash_placeholder requirement with input-addressed provisional + blake3 marker strategy ✅ 2m (started: 2026-04-05T10:03Z → completed: 2026-04-05T10:04Z)
- [x] persistent-pathinfo/spec.md: replace filesystem existence cache check with castore content probe, reference castore-store spec ✅ 1m (started: 2026-04-05T10:04Z → completed: 2026-04-05T10:04Z)
- [x] nickel-stdlib/spec.md: change Sandbox default from 'wasm to 'native, reference portability spec rationale ✅ 1m (started: 2026-04-05T10:04Z → completed: 2026-04-05T10:05Z)
- [x] fetchers/spec.md: replace "continue the build" with "exit with re-run instruction" and explain why ✅ 1m (started: 2026-04-05T10:05Z → completed: 2026-04-05T10:05Z)
- [x] architecture/spec.md: add crunch-build to crate layout table ✅ 1m (started: 2026-04-05T10:05Z → completed: 2026-04-05T10:05Z)

## Phase 2: Verify consistency

- [x] Read each updated spec end-to-end and check no internal contradictions remain ✅ 2m (started: 2026-04-05T10:06Z → completed: 2026-04-05T10:07Z)
    - Fixed: nickel-eval spec still said "closed record contract" — updated to "open"
    - Fixed: defaults spec said --fix "retries" — updated to "exits, instructing re-run"
    - Noted: store-dir spec references removed functions (all_outputs_exist, path_exists_on_disk, load_cached_outputs) — separate issue, out of scope
- [x] Cross-check updated specs against AGENTS.md — ensure no conflicts ✅ 1m (started: 2026-04-05T10:07Z → completed: 2026-04-05T10:07Z)
- [x] Verify no other specs reference the old contradicted text ✅ 1m (started: 2026-04-05T10:07Z → completed: 2026-04-05T10:07Z)
