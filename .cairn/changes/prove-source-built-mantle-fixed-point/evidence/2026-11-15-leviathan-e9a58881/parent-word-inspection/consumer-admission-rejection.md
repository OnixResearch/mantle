# Consumed deferred Rust invocation rejection

## Subject and decision

Frozen Mantle code `e9a58881b9ff491f47864632367c9b8ea4af76e3`, complete source tree `e62509cd40ab6e4b202dce425a40483d74101210c6322c64f70b196ff7e177f7`, and prepared manifest `3096052305ed9bc2cf0b23b3291dc98153f5b090d7f9abe726368638f883a946` are **held: do not launch this cohort**. Source-only readiness and full profile transfer equality did not exercise deferred request admission.

The trusted inert `scan.py` inspected all six current physical recipe identities from `inventory.json`: 908,012 direct words / 1,042,074 parts had no context-free lexical/shape failures. It also inspected 602,749 words in statically decodable JSON data; no deeper encoded JSON candidates were skipped. That separate data inspection found the consumed Rust invocation defects below. Direct-word success is explicitly not a complete typed-scope or launch verdict.

| Encoded word class | Occurrences | Actual source-selected operand |
| --- | ---: | --- |
| Literal filesystem spelling in linker data | 2,927 | `-Clink-args=-Wl,-rpath,$ORIGIN/../lib` |
| Literal filesystem spelling in cfg data | 4 | `feature="regex-syntax/default"`, `feature="valuable/std"`, `feature="valuable?/std"` |
| Noncanonical typed Input member | 12 | Original proc-macro `../../library/proc_macro/src/lib.rs` or im-rc `./src/lib.rs` manifest paths |

`trace-encoded.py` matched all 2,943 flags, across 16 exact word patterns and 2,933 producer templates. Every flagged template has an immediate Produced-input consumer. `encoded-trace.json` retains every producer ID, candidate coordinate, distinct word and direct consumer link. Its invocation-local executable/argument fields are null because Exec owns those fields at operation level; the producer/consumer IDs are the complete scope of that immediate-link trace.

## Actual unchanged admission path

- `source_built_rust_programs_translation.rs::bind_tmp_step` moves the original command arguments/environment into a RecipeInvocation, serializes it into a WriteText product, binds that product as the Exec's `invocation_input`, and empties the original static operands. No source-owned JSON transformer runs between that frame and its compiler command.
- `source_built_rust_programs_successors.rs::compiler_exec` likewise serializes its supplied compiler arguments/environment into a WriteText invocation and binds it directly as the compiler Exec's request, without a helper rewriting those words.
- `source_built_recipe_exec.rs::invocation` reads that whole authenticated Text and calls `source_built_recipe::decode_invocation`.
- `decode_invocation` checks canonical JSON and every argument/environment/cwd/descriptor word through `validate_word_scoped`. Literal slash operands and noncanonical Input members are still rejected before the executable is launched.
- The observed Rust recipe byte identity is `2401c71c1efe2053251f9e65021b4a9ec16fcc98359f5b4536359ca75c01a407`; frozen `source_built_recipe.rs` is `28ca15f8081b88c5747a851a1474d92277fba142f0b7fdfa1f74d8a48374d009`.

## Executed regression, not only a mirrored inspector

Local Pueue task **26** launched the actual Rust test on Leviathan from an isolated copy of the frozen product sources plus the new regression registration/test. It completed with exit **101**. The exact command, task status and log are `reproduce-profile-admission.sh`, `reproduction-pueue26.json` and `reproduce-profile-admission-before.log`. No proof, provider or profile helper was executed by this reproduction.

`original_rpath_profile_survives_strict_deferred_invocation_admission` passes the real source-selected compiler profile arguments through the actual canonical deferred invocation decoder. Its baseline failure is:

`Build("source recipe: untyped filesystem path in literal command word for step profile.compile")`

Captured execution: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2970 filtered out; finished in 0.00s`. The unrelated Nix SSH/default-ref warnings in that log did not prevent compilation or test execution; subsequent verification uses the same explicit Git HTTPS environment as the existing source qualification procedure.

## Repair and remaining authority

The author-only repair types the genuine minicargo feature values and successor rpath as LiteralData, keeping their original bytes. Compiler source target members use the existing archive-relative `resolve::normalize` before Input construction; absolute or above-root paths remain errors of that resolver. The original manifests and consumer/validator remain unchanged. Input constructors accept the owned normalized String, avoiding an extra path copy. No new addressing policy or relaxed literal/path admission is introduced.

Maintenance owner: Mantle's minicargo/successor source-program authors. Durable capability: source-selected compiler data and archive-relative source targets remain admissible without changing consumer authority. The real-profile/deferred-decoder regression plus adjacent positive/negative source-member cases provide repeatable coverage.

After that exact smoke checkpoint, temporary formatted paths and consumed path-loop Strings were migrated into the owned-path constructors without an extra copy. Final committed-source qualification must cover this caller cutover; the five-file smoke hashes are not relabeled as final-commit evidence.

Remote Pueue **1585** completed Success: the repaired original-profile/deferred-decoder regression and the unchanged path/name/control rejection each executed one test and passed. Exact output is in `verify-template-repair.log`; metadata is `verification-pueue1585.json`. Its mistyped file-name resolver filter selected zero tests and is not resolver evidence. Correct module/boundary execution in **1586** then passed **23 tests**, including the new sibling/dot and archive-escape/absolute cases: `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 2949 filtered out; finished in 0.00s`. Exact output is `verify-source-resolver.log`, with metadata `resolver-verification-pueue1586.json`.

The requested Luna graph reviewer independently returned **REJECT frozen e9a58881**, with the exact 2,933 Produced-Text request/unguarded-Exec pairs and the same typed-word blocker. Its bounded report is `../graph-review/consumer-review.md`. Neither that rejection nor the partial source identity pass is a complete review of a future subject.

A new committed exact source, source-bound qualification, source-only regeneration, complete current profile transfer, complete requested current Luna review and genuine protected stages/reconciliation are still required. None of the old or partial review passes authorizes a future subject. I3-I5, proof-dependent V2/V3, native parity and release acceptance remain open.

The e9 Luna native expansion report also records one declared make derivation-output edge attached only to an inert specs-query target. `validate_node_program` checks retention of original Nix `input_derivations` references, not proof that a command read every declared dependency. The report therefore does not establish actual make execution for that edge. No recipe, pin or policy was changed to hide that distinction; complete review of the new subject must retain the declared-edge/actual-operand claim boundary.

Follow-up source tracing distinguishes that whole-output-tree reference from the actual executable member: `source_built_native_programs_host.rs::linux_headers` selects the same owning Make program's `members["out"]["bin/make"]` and plans both `make-version-request` and `original-make-version` Exec leaves with that Produced member as executable authority (lines 812–857). A whole-Tree-reference-only scan therefore does not enumerate the genuine member uses. The new complete review must validate that member producer's ownership/closure rather than infer an absent Make dependency from the inert Tree anchor. These are source/planned-graph observations, not evidence that Make executed in a proof.
