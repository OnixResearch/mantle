# Root cause: Cargo Git dependency references lose reachable packages

## Validated result

Mantle's native lockfile matcher rejects Cargo's source-qualified Git dependency references. The dependency reference omits the resolved `#commit` suffix. The corresponding package source includes that suffix.

`lock_dependency_matching_keys()` compares these strings for exact equality. The comparison rejects both locked `artifact-auth-core` revisions. This removes required packages before Mantle constructs its unit graph.

This is a planner defect, not a Rust compilation failure. The original stage-1 metadata records zero units and no stage-2 attempt.

Owner: Mantle native Rust planning in `src/rust_plan.rs`.

## Exact source and attempt

- Runtime source: `97f47ae2a644651734324f44991cd4cae847499a`.
- Runtime binary BLAKE3: `3a8e1b46c634cb31f44b8f63ecd9d675dce2a1c236d051b91a938697e2f8f40c`.
- Remote host: `leviathan.cymric-daggertooth.ts.net`.
- Run root: `/home/brittonr/mantle-runs/dev-resume-20260904`.
- Preserved attempt: `.dev-cold-97f47ae2.source-built-fixed-point-staging-3039000` under the run root.
- Separate diagnostic directory: `root-cause-20260905` under the run root.

The replay checked the runtime binary against its expected BLAKE3 digest. `source-binding.log` checks six inspected files against their Git blob identities at the runtime commit. Git blob hashes serve Git interoperability only. Diagnostic content hashes use BLAKE3.

## Causal chain

1. `Cargo.lock:158-180` contains two packages named `artifact-auth-core`, both at version `0.1.0`. Their sources select revisions `c932138d...` and `e41340be...`.
2. Dependency references at `Cargo.lock:1371`, `1416`, `1702`, `4916`, and `8862` include the Git URL and `?rev=...`, but omit `#commit`.
3. `parse_lock_dependency_fact()` at `src/rust_plan.rs:2821` preserves that source string.
4. `lock_dependency_matching_keys()` at `src/rust_plan.rs:2369-2378` compares the dependency source against the package source without normalization.
5. `native_reachable_lock_source_cargo_packages()` at `src/rust_plan.rs:2294` removes both unmatched package identities from the reachable source set.
6. Native package planning reports five `missing-captured-git-source-fact` blockers. The affected consumers are `valence-core`, `crunch-action-result-core`, `crunch-build`, `crunch-release-core`, and `mantle`.
7. `summarize_unit_derivation_graph_with_native()` and `blocked_unit_derivation_graph_from_native()` at `src/rust_plan.rs:10426-10495` return an empty blocked graph.
8. `execute_topology_rust_plan()` at `src/cli_application.rs:4971` starts action admission before it reports that graph's existing blockers.
9. `SourceBuiltRustActionRuntime::start()` at `src/source_built_rust_action_shell.rs:75` asks for supported execution units. `combined_topology_indices()` at `src/rust_plan.rs:12171` finds none and returns the secondary `missing-supported-unit` error.
10. The early error prevents the original command from printing its planning receipt. Thus `stage1/receipt.json` is empty and stderr hides the earlier dependency rejection.

## Reproduction and controls

The diagnostic reused the original binary, staged source, target triple, and receipt-bound rustc wrapper. It omitted execution and action-admission flags. It used an absent Cargo executable and separate output/state paths.

The planning-only replay preserved the original package failure. It reported five missing Git-source facts and zero derivations. Registry source planning remained ready. No execution directory appeared.

Two small fixtures used the same binary and provider. Their source files, manifests, and vendor data are identical. Only one dependency reference in their fixture lockfiles differs:

- `fixtures/qualified`: a source-qualified Git reference without `#commit`. The graph is blocked and has zero derivations.
- `fixtures/unqualified`: a bare dependency name. The graph is ready and has two derivations, with no blockers.

The control has exactly one Git dependency. A bare name is not a repair for the real lockfile's two-source ambiguity.

The deterministic receipt checks returned:

```text
root-cause verification: PASS
original source: five missing-captured-git-source-fact blockers; zero derivations
qualified Git dependency reference: rejected; zero derivations
unqualified control: ready; two derivations; zero blockers
```

The CLI exits zero for planning-only output even when the receipt says `ready: false`. The verification therefore checks receipt fields, not process success alone.

Evidence files are `plan-only.json`, `qualified.json`, `unqualified.json`, their stderr/status files, `preflight.json`, `meta.json`, and `original-stage1-stderr.txt`. The full remote receipts remain outside the preserved attempt.

## Why the earlier checks missed this

The vendor repair validated source availability through Cargo and the host-tool-free vendor guard. Its native-planner tests checked directory discovery and malformed configuration.

The inspected end-to-end Git test at `src/rust_plan.rs:25135` uses a bare lockfile dependency name. The parser test at `src/rust_plan.rs:24371` includes `#commit` in its fixture dependency reference. Neither fixture exercises the real mismatch.

The failed workflow reaches native package planning at Mantle stage 1, after the compiler bootstrap. Its provider preflight does not establish readiness of this package graph.

## Repair boundary

Normalize Cargo dependency-reference identity before matching it to a resolved package identity. Preserve the URL, revision selector, and final resolved commit. Distinct revisions must remain distinct.

Report existing planning blockers before action-runtime startup. Add the real source-qualified edge shape to positive and negative regression tests. Run a native planning-only preflight before the long bootstrap.

Do not change the real lockfile, merge the two Artifact pins, remove authority checks, or accept an empty graph. The separate source-to-vendor-directory binding also needs verification during repair.

## Approach registry and bounds

| Family | Result | Evidence |
|---|---|---|
| Source admission | Validated root cause | Raw source comparison and same-binary controlled replay |
| Target selection | Rejected as the initiating cause | The same target and source layout produce two units in the control |
| Graph composition | Validated secondary diagnostic defect | Existing blockers become an empty graph, then action admission masks them |

The serial analysis passes are correlated. The investigation used three planning replays within the four-replay budget. Each replay had a five-minute limit. The terminal reason is a validated causal result, not budget exhaustion.

## Limits and preserved inspector errors

No production source, production lockfile, original proof input, provider, cache, or original audit changed. The bootstrap did not restart. This investigation does not prove a corrected full graph, a successful self-build, provider correctness, or release eligibility.

The first receipt-verifier attempt used an unsupported two-argument Steel `assert!` call. The next attempt used structural equality for JSON numeric values. The final verifier uses explicit errors and numeric comparison. These inspector errors did not change any replay input or receipt.
