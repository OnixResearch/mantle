# Audit: Freshness Probe and Refresh Surface

Audited against `r[project_workflows.freshness_probes]` and
`r[project_workflows.freshness_probe_refresh]` requirement clauses, and the
change's own `r[project_workflows.freshness_probe_proof_rail]` requirement.

## Source Files Audited

- `crates/crunch-project-core/src/freshness.rs` (1,116 lines)
- `crates/crunch-project-core/src/refresh.rs` (880 lines)
- `crates/crunch-project-core/src/soundness.rs` (1,230 lines)
- `crates/crunch-project/src/refresh_adapter.rs` (1,170 lines)
- `src/project_cmd.rs` (380 lines)
- `src/project_resolve.rs` (120 lines for freshness probes)
- `tests/project_refresh_cli.rs` (741 lines)

## Probe Family Coverage

### Built-in Git Reference
- ✅ `FreshnessProbe::GitRef` defined with `repository` and `reference` fields.
- ✅ `observe_git_ref()` calls `git ls-remote`, parses output for matching ref.
- ✅ `parse_git_ref_probe_output()` validates hex rev length.
- ✅ Test: `freshness_git_ref_probe_reports_stale_without_mutating_lock` covers
  stale reporting, no-mutate assertion, and `--no-network` mode.

### HTTP Text
- ✅ `FreshnessProbe::HttpText` with URL validation.
- ✅ `observe_http_text()` reads URL content, validates UTF-8.
- ✅ `trim_freshness_text()` normalizes whitespace.
- ✅ Bounded reader: `read_url_limited()` enforces byte limit.
- ✅ Test: `list_stale_reports_stale_and_failed_without_mutating_files` covers
  HTTP text probe in list-stale context.

### HTTP JSON
- ✅ `FreshnessProbe::HttpJson` with URL and JSON pointer.
- ✅ `observe_http_json()` selects by pointer, renders to string.
- ✅ Test: `freshness_http_json_template_refreshes_selected_stale_input` covers
  HTTP JSON probe with template rendering and selected refresh.

### Local File
- ✅ `FreshnessProbe::LocalFile` with path.
- ✅ `observe_local_file()` returns BLAKE3 hash of file content.
- ✅ Test: `list_stale_reports_stale_and_failed_without_mutating_files` covers.

### Local Directory
- ✅ `FreshnessProbe::LocalDirectory` with path.
- ✅ `observe_local_directory()` walks tree, BLAKE3-hashes sorted relative paths + content.
- ✅ Bounded: `FRESHNESS_LOCAL_TREE_MAX_ENTRIES`, `FRESHNESS_LOCAL_TREE_MAX_BYTES`.
- ✅ Test: `freshness_local_directory_probe_updates_lock_digest`.

### Command (bounded)
- ✅ `FreshnessProbe::Command` with `CommandFreshnessProbe` fields: `argv`, `cwd`,
  `env`, `timeout_ms`, `output_limit_bytes`, `success_statuses`, `utf8_required`.
- ✅ `spawn_freshness_command()` enforces bounds.
- ✅ `wait_for_freshness_command()` applies timeout.
- ✅ `collect_command_output()` enforces output_limit_bytes and UTF-8.
- ✅ Test: `freshness_command_missing_timeout_and_output_limit_fail_deterministically`.

## List-Stale / Refresh Behavior

### `list_stale` is no-mutate
- ✅ `list_stale()` in `crunch_project_core::refresh` never writes files.
- ✅ `list_stale_with_options()` in refresh_adapter calls core `list_stale()`.
- ✅ Test: `list_stale_reports_stale_and_failed_without_mutating_files` asserts
  no file changes.

### `refresh` updates only stale inputs
- ✅ `refresh_inputs()` filters to stale decisions before resolving.
- ✅ `should_resolve_input()` gates on `FreshnessDecisionKind::Stale`.
- ✅ Partial failure: `refresh_partial_failure_writes_successes_and_exits_nonzero`.
- ✅ Exits non-zero on any failure.

### No-network mode
- ✅ `--no-network` flag on `list-stale` and `refresh` commands.
- ✅ `collect_freshness_observations()` passes `no_network` to resolver.
- ✅ `observe_freshness_probe()` returns `FreshnessObservationStatus::NetworkRequired`
  for network-requiring probes when `no_network`.
- ✅ Test: `freshness_no_network_mode_does_not_contact_http_probe`.
- ✅ Test: `freshness_git_ref_probe_reports_stale_without_mutating_lock` with
  `--no-network` asserts git operations not performed.

### Soundness checks
- ✅ `check_project_soundness()` in soundness.rs covers freshness probe failures.
- ✅ Static mode is no-network by default.
- ✅ `ProjectSoundnessClass::FreshnessProbeFailure` diagnosed.
- ✅ Non-claims: `NON_CLAIM_SOURCE_AVAILABILITY`, `NON_CLAIM_INPUT_TRUST`.

## Gaps for Proof Rail

| Feature | Code Coverage | Composed Proof Rail |
|---|---|---|
| Built-in Git probe | ✅ | ❌ |
| HTTP text probe | ✅ | ❌ |
| HTTP JSON probe | ✅ | ❌ |
| Local file probe | ✅ | ❌ |
| Local directory probe | ✅ | ❌ |
| Command probe | ✅ | ❌ |
| list-stale no-mutate | ✅ | ❌ |
| refresh selected-stale | ✅ | ❌ |
| check no-network | ✅ | ❌ |
| Full composition (probe→list-stale→refresh→check) | ❌ individual tests | ❌ |
| Versioned evidence record | ❌ | ❌ |
| Network-required in no-network as fixture assertion | ✅ unit | ❌ composed |
| Command oversize/invalid-UTF-8 as fixture assertion | ✅ unit | ❌ composed |
| list-stale no-mutate asserted on lock/gen-inputs/roots | ✅ partial | ❌ full set |

## Summary

All 6 probe families are implemented and individually tested. The `list-stale`,
`refresh`, and `check` CLI commands work with `--no-network` mode. The gap is
a single composed offline proof rail that:
1. Exercises all probe families through the full flow
2. Emits a versioned, redacted, non-overclaiming evidence record
3. Asserts cross-feature behavior (list-stale no-mutate across all state files,
   refresh selected-stale only, check no-network)
4. Asserts negative cases in a composed context