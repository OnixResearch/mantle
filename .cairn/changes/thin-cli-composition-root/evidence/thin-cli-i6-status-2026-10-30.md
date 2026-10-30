# thin-cli-composition-root: I6, V1, V2 slice status (2026-10-30)

Slice report for tasks I6, V1 and V2, produced on branch `agent/thin-cli-i6-20261030` (base `177661759`).
It records what was implemented and what was executed. It does not accept I6, V1 or V2, tick any checkbox, or
replace the V3/V4 observations in `command-parity.md` and `architecture-validation.md` (commit `5fc2ad4b3`).
Every `test result:` line below is copied from a captured log under
`/home/brittonr/.cargo-target/agents-20261030/thin-cli/scratch/logs/` (named per row). All cargo commands ran
through `run-in-worktree-v1.py` with `--locked --offline` and `-- --test-threads 1`.

## Status table

| Task | Status | Evidence |
|---|---|---|
| I6 | **partially delivered.** Audit tool and report delivered; one real gap (3 `foreign-import produce-*` leaves) closed in source; no other leaf lacks a reachable contract classifier except 3 dispositioned leaves. Per-leaf test attribution is heuristic and incomplete (see Limits), so I6 operation completeness is not established by this slice. | `i6-coverage-audit-2026-10-30.{md,json}`, `i6-coverage-dispositions-2026-10-30.json` |
| V1 | **partially delivered.** 25 new tests in four new integration files plus two unit tests; success and failure boundaries for `profile`, `remote gateway` (5 operations), `cargo-build-bundle`, `release gauntlet`, and producer classification. Not every V1 category exists for every command family; remaining candidates are listed under Not delivered. | rows 3-7 below |
| V2 | **already covered, one hardening fixture added.** Every V2 authority class had a rejecting fixture in both scopes; a table-driven fixture now makes every forbidden-table row load-bearing. | rows 8-9 and the V2 section |

## Commands and results

| # | Command | Exit | Result line (verbatim) |
|---|---|---|---|
| 1 | `python3 scripts/audit-i6-coverage.py --self-test` | 0 | `i6-coverage-audit self-test: PASS` |
| 2 | `python3 scripts/audit-i6-coverage.py --handoff <handoff> --dispositions <dispositions> --out-json <json> --out-md <md>` | 0 | `{"flag_counts": {"classified-via-shared-dispatcher": 24, "failure-path-classifier-level-only": 31, "no-classification-reachable": 3, "no-failure-path-test": 11, "no-success-path-test": 36, "report-before-classification-advisory": 1}, "gap_leaves_without_disposition": 39, "leaves": 187, "leaves_with_gap_flags": 42}` (`final-audit.out`) |
| 3 | `cargo test --locked --offline -p mantle --test gateway_cli --test profile_cli -- --test-threads 1` | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s` (gateway_cli); `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.49s` (profile_cli) (`t-new.log`; identical counts again in `t-new-rerun.log`) |
| 4 | `cargo test --locked --offline -p mantle --test cargo_build_bundle_cli --test release_gauntlet_cli -- --test-threads 1` | 0 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s`; `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s` (`t-new2.log`) |
| 5 | `cargo test --locked --offline -p mantle --test foreign_import_cli -- --test-threads 1` (parity for the changed producer path) | 0 | `test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 11.86s` (`t-foreign.log`) |
| 6 | `cargo test --locked --offline -p mantle --bin mantle foreign_import_cmd::tests -- --test-threads 1` | 0 | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2838 filtered out; finished in 4.50s` (`t-unit-foreign.log`; includes the 2 new unit tests) |
| 7 | `cargo -Zscript scripts/check-cli-architecture.rs --self-test` | 0 | `cli-architecture self-test: PASS` (`t-arch-self.log`) |
| 8 | `cargo -Zscript scripts/check-cli-architecture.rs` | 0 | `cli architecture: PASS` (`t-arch.log`) |

| 9 | `cargo fmt --check -p mantle` (after `rustfmt --edition 2024` on the five touched files) | 0 | no output (`t-fmt2.log`: `exit=0`) |
| 10 | `cargo clippy --locked --offline -p mantle --all-targets --no-deps -- -D warnings` | 0 | `Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 17s` (`t-clippy.log`; the only warning line is cargo's pre-existing duplicate-bin-target manifest notice). Run on the tree after `rustfmt`; the audit report predates that formatting. |

The one ignored test in row 5 is the pre-existing `#[ignore]` in `foreign_import_cli`; it did not run. Row 5 and 6 ran
before `rustfmt` was applied to the touched files (formatting only); they were not re-run afterwards.

## I6: what the audit does and what it found

`scripts/audit-i6-coverage.py` (BLAKE3 `b2fa1f5b8feffe1461316f9b643b0b82a780f11900e5c99182cc06c97800bf30`) is a
deterministic source-text audit. It derives the leaf set from the Clap `Subcommand` enums, cross-checks it against
`config/operator-command-catalog.json` (BLAKE3 `604ae2424861e1d9657774f89f0ef5aa7df43ac2724f7f612173ed3861169341`),
finds each leaf's dispatch arms, builds a name-resolved call graph over `src/**`, and asks whether any contract
`classify_*` function (all 13 `pub fn classify_*` of `mantle-application-contract`, derived from source) is reachable
from the leaf's dispatch, through which chain, whether the verdict is discarded and whether a report is emitted
first. Self-test fixtures (positive and negative): classified-before-report leaf; no classifier; verdict discarded;
print before classify; classified but no failure test; classifier only in a comment, a string or a `#[cfg(test)]`
module; classifier only behind a host-context module; classification by the caller of a port method (positive and
negative); a lookalike method on an unrelated type; a `Command::new("cargo")` invocation that is not a leaf test; a
leaf invoked across consecutive `.args` calls; catalog drift; and stale, unknown and malformed dispositions.

Findings on source `HEAD`+working tree of this branch:

- **Inventory.** The catalog has 211 entries and **178** terminal leaves, not 154. The handoff's 154 are a strict
  subset; the 24 extra terminals are exactly the handoff's "outside the checked catalog" rows
  (`bootstrap trust-report`, `test`, 7 `profile *`, 4 `cargo-build-bundle *`, 11 `remote gateway *`). The catalog's
  `identity_blake3` is `5d72a441...`, not the handoff's `12e74923...`; the catalog was regenerated after the handoff
  (last catalog commit `125774379`). The Clap tree yields 185 leaves: all 178 catalog terminals plus 7 hidden or
  no-subcommand surfaces (`__operator-contract`, `bootstrap` with no action, `bootstrap source-built-replay`,
  `evaluator-worker`, `evaluator-worker-fixture`, `remote usage-executor`, `remote-secret-worker`); two flag-mode
  surfaces are audited through their parent (`remote serve --binding metadata`, `self-build ... --prepare-source-programs`).
  187 rows in total. No catalog path is missing from the Clap tree.
- **Classification reach.** 184 of 187 leaves reach a contract classifier from their own dispatch arm (107 directly,
  53 indirectly) or through the shared dispatcher that executes the leaf (24: the 4 `cargo-build-bundle`, 6 `profile`,
  3 `release gauntlet`, 11 `remote gateway`). The remaining 3 have no reachable classifier and carry reviewer
  dispositions anchored to source: `refactor list` (pure, no effect) and the two hidden `evaluator-worker*` child
  protocol endpoints (the spawning parent classifies the worker exit; this is not a contract classifier call).
- **Real gap found and closed.** `foreign-import produce-aterm`, `produce-backend` and `produce-nix` read their
  closure inputs, wrote two artifacts and printed a success report with **no effect plan and no observation**:
  `foreign_effect_plan` returned `None` for them. This contradicts the handoff's key FO. Fixed (below).
- **Advisory.** `nix-free-demo generate` is flagged `report-before-classification-advisory`: the lexical rule sees
  the early `emit_rejected_generation(...)` return before the classifier. That wrapper itself classifies first
  (`classify_nix_free_demo_rejection`), so this is a heuristic false positive; the script's wrapper check does not
  resolve it. Recorded, not suppressed.

### Gap closure (behaviour-preserving)

`src/foreign_import_cmd.rs`: the three producers now plan `foreign-input-read` (1 call) and one
`foreign-producer-artifacts` write (identity = output directory), run the existing `foreign_finish_effects`
observation machinery, and classify before `emit_report`. A success report requires both artifacts to read back as
non-empty regular files; a rejection must have published nothing. Messages, streams and exit classes are the
producers' own; `foreign_import_cli` (18 passed) is the parity run. Unit tests
`an_accepted_producer_report_requires_both_artifacts_to_read_back` and
`a_rejected_producer_report_is_consistent_only_when_nothing_was_published` cover accepted-with-missing/empty artifact
(Err), no publication (Err), rejection after a read and after an unreadable input (Ok).

## V1: tests added

| File | Tests | Categories |
|---|---|---|
| `tests/profile_cli.rs` | 6 | valid mapping and JSON output per operation (`list`, `generations` report their own operation), invalid DTO (relative target, non-canonical generation, unknown subcommand), domain blocker (exit 1, no report), refusal before any filesystem effect |
| `tests/gateway_cli.rs` | 7 | JSON output; determinism; digest bound to generation, opcode and byte bound; zero generation, non-store path, unknown opcode refused; exposed header refused; quota provisioning idempotent and ledger preserved; missing directory refused without creating it; fault campaign deterministic per seed and refusing to reuse a directory |
| `tests/cargo_build_bundle_cli.rs` | 8 | `inspect` success (manifest printed, identity matches), compatibility mismatch, tampered manifest, missing and malformed inputs (adapter fault), usage errors; `verify` trust-policy mismatch, absent payloads, missing trusted key |
| `tests/release_gauntlet_cli.rs` | 4 | malformed, wrong-shaped and missing reports refused with no published file; unknown kind and missing evidence are usage errors; aggregates refuse missing evidence without writing a report |
| `src/foreign_import_cmd.rs` tests | 2 | wrong observation (accepted report whose artifacts are absent or empty), rendering of expected failures |

None assert diagnostic wording; they assert exit class, stdout presence/absence and filesystem effects.
Capability-gating: none of the new tests needs bwrap, a store or the network, so none can skip.

Not delivered for V1: `remote gateway read`/`serve`/`nix-once` and other service-socket operations (need a real
owner peer; the existing ones are `#[ignore]`d), `cargo-build-bundle` `verify` success, `export` and `replay`
(need signed archives), a positive-path `profile install`/`switch`, `bootstrap elf-relocate`, and the `mantlepkgs`
leaves the audit lists without an attributable success test. The "missing dependency" and "rendering failure" rows
are covered by existing tests (`operator_diagnostics`, `/dev/full`), not by new ones. No test was mutation-checked
by changing production code and rebuilding; only the architecture fixture was (see V2).

## V2: coverage assessment

`architecture-validation.md` maps the V2 classes to lexical self-test families; re-reading `fn self_test` confirms
each class has at least one rejected-authority fixture asserted in **both** the core and the contract scope, plus
accepted-authority fixtures (alloc/serde/pure alias/test-only helpers) and adapter/integration-test acceptance. CLI,
Snix, filesystem, process, async runtime, environment, clock, random, network, provider, store service and rendering
are all present. The gap was that each class was sampled by one or two fixtures while `CONTRACT_FORBIDDEN` holds 71
rows; deleting an unsampled row (for example `smol`, `ureq`, `crate::stagex_provider`) would not have failed the
self-test. Added: a table-driven fixture that writes one authority reference per table row and requires its rejection in
both scopes, and a required-row list for each of the 12 V2 classes. Mutation check (captured interactively, not
in a retained log): with `"smol"` removed from `CONTRACT_FORBIDDEN` the self-test printed
`cli-architecture self-test: FAIL: async runtime authority lost its forbidden-table row `smol``. Limit: a row
removed that is not in the required-row list (for example `rustix::net`) is still not detected.

## Limits and non-claims

- **Test attribution is lexical and unreliable in both directions.** CLI tests are matched by literal arguments
  across helper calls; unit tests by calls into leaf-exclusive code. Spot checks show false negatives under the final
  matcher (for example `attest show` has `tests/attest_cli.rs:734 attest_show_and_verify_artifact`, but the final
  run attributes no test to it), and an earlier looser matcher over-attributed. The 36 `no-success-path-test` and
  11 `no-failure-path-test` flags are therefore *candidates for review, not proofs of absence*, and 31
  `failure-path-classifier-level-only` rows mean only a contract-level classifier failure test was found.
- Reach evidence is a lexical call graph over name resolution (ambiguous or std-lookalike method names are dropped,
  host-context modules are excluded); "classifier reachable" does not prove it runs before the report on every
  path, and the ordering rule only sees emissions that precede the classifier inside its owning function.
- Nothing here runs a classifier against a real operation outside the tests listed above. Acceptance, tick-boxing,
  V3-V5 reruns on the final source, `nix flake check` and the Cairn gates were not run by this slice
  unless a row above says so.
- One slip, corrected: an early `edit` of `scripts/check-cli-architecture.rs` addressed the canonical checkout
  (`/home/brittonr/git/OnixResearch/mantle`) instead of this worktree. It was undone by removing exactly the 47
  inserted lines; `git diff -- scripts/check-cli-architecture.rs` there was empty afterwards. The same content was
  then applied in this worktree. The canonical checkout holds other uncommitted changes that are not mine.
