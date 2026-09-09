# Promoted authority review

## Question

Does `promoted-cold-a7841aea` select fresh provider and output authority without dev-cache, resume, or checkpoint inputs?

## Inspected evidence

- `promoted-cold-a7841aea.command.txt` binds the source commit, executable, source profile, lineage, native provider identity, and resource limits.
- `promoted-launch-a7841aea/argv.txt` records the actual arguments. They omit every dev-cache, resume, fast-fail, and checkpoint option.
- `promoted-launch-a7841aea/plan.json` records plan `ddb30a9b18d0015110cde3cc69003403deeb71d8b706b803be38795f554ec6f7`. Its policies forbid provider-cache completion, live fetches, Cargo calls, ambient discovery, and fallback.
- `promoted-launch-a7841aea/host-facts.txt` records the launch checks and declarations. The empty-authority declaration alone is not evidence of isolation.
- `promoted-launch-a7841aea/current-observation.txt` records the live process and distinct, nonsymlink `native-store` and `native-state` directories at `2026-09-08T18:13:26-04:00`. Both success aliases were absent.
- The source review covered `src/cli_application.rs`, `src/source_built_fixed_point_shell.rs`, `src/source_built_fixed_point_shell/dev_resume/load.rs`, and `src/source_built_fixed_point_shell/checkpoint_integration.rs`.

## Source-path findings

1. The CLI passes the explicit optional cache and checkpoint paths into `SourceBuiltFixedPointOptions`. It passes the resume and fast-fail flags separately.
2. `prepare_attempt` uses `fs::create_dir` for the attempt-local native store and state when the cache option is absent. It imports source authority into that fresh state. It does not call `seed_dev_store_snapshot` on this branch.
3. `prepare_dev_resume` returns the cold plan before candidate reads when resume is false. `restore_constructed_provider_checkpoint` returns before store access when the checkpoint path is absent.
4. `dev_cache_adoption` returns false before cache access when the cache path is absent. The native-prefix and checkpoint-import paths also require explicit options.
5. `native_build_command` clears the child environment. It passes the attempt-local store and state, strict hermeticity, offline source preflight, and no substitution.
6. `run_attempt` constructs providers and runs the fixed point before `finish_promoted_attempt`. Only that path reaches promoted receipt publication.

These code facts support the selected cold path. They do not replace terminal execution evidence or prove operating-system prevention of every unrelated file read.

## Decision

The recorded arguments and source select the fresh promoted path. The plan review accepts that selection, not proof completion.

The dev and promoted plans have the same digest. That digest does not select cache authority. The explicit options select that authority.

This review occurred after launch. It is not retroactive evidence of a prelaunch review. No initial directory census or per-open filesystem audit was captured. The empty-state argument relies on the create-new source path plus the recorded arguments and later directory observation.

The runtime source remains `a7841aea055bf643cc473164b98a2442014f080b`. Commit `e5cf7fdb` changed import order and assertion layout in three Rust files. The complete diff was inspected. A result for the runtime cohort does not prove the newer source bytes or cross-cohort binary equality.

## Review method and limits

The independent read-only review, Pueue task `4564`, timed out after 240 seconds with no output. Its exit code was 124.

The remaining source and observation reviews were local, correlated passes. The search budget allowed one worker, 18 targeted worker reads, and one bounded local source review. No review authorized a second proof or a change to active inputs.

| Family | Mechanism | Result | Remaining check |
|---|---|---|---|
| Explicit selection | Absent cache and checkpoint options select cold branches before reads | Source path accepted | Inspect terminal origin and adoption fields |
| Fresh authority | Create-new store/state plus explicit child paths | Source argument supported by live directory observations | Inspect all promoted stage receipts |
| Receipt publication | Successful staging moves to the final output before alias publication | Collector path corrected from source | Verify the final receipt and both aliases |
| Independent audit | Separate read-only worker | Blocked by timeout | No independent review result is claimed |

The source-path claim is narrower than runtime acceptance. The terminal runtime verdict remains pending.

## Owner and next action

Owner: Mantle maintainers and the operator for `add-dev-cache-cross-run-resume`.

After the remote wrapper records termination, collect the final receipt, stage receipt identities, fixed-point metadata, and alias targets. Reject adoption or restored-stage evidence on the promoted cold path. Keep V3 open until those observations pass review.
