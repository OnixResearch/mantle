Evidence-ID: no-std-functional-core-v3-project-shell-boundary
Task-ID: V3
Artifact-Type: verification-note
Covers: architecture.nostd.core.crate.boundary.effectful.dependency.outside, functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads, functional.core.shell.adapters.effect.translation.project.refresh.io.in.shell
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

Validation command:
- `cargo test -p crunch-project shell_adapter_keeps_refresh_io_outside_core`
- Output: `test refresh_adapter::tests::shell_adapter_keeps_refresh_io_outside_core ... ok`
- Output: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s`

Code inspection confirmed:
- `crates/crunch-project/src/refresh_adapter.rs` still owns `RefreshResolver`, git/url/local-file hashing calls, and translation into owned `RefreshInputsRequest`, `ApplyOutcomesRequest`, and related core request types
- `crates/crunch-project/src/attestation_adapter.rs` translates borrowed manifest/lock/root inputs into owned `ProjectAttestationRequest` before calling `crunch-project-core`
- `crates/crunch-project/src/upgrade_adapter.rs` keeps std error translation outside the core crate, while `crates/crunch-project/src/lib.rs` is only module wiring + re-exports
- `src/project_cmd.rs` remains the root CLI shell for manifest/lock file I/O, generated-input writes, and user-facing formatting
- `openspec/changes/no-std-functional-core/evidence/ownership-review.md` now classifies every touched std workspace source file outside the legacy paths, including `crates/crunch-project/src/lib.rs` and `src/project_cmd.rs`

Result: project refresh and project-file I/O remain in shell code, and the `crunch-project-core` boundary stays on owned normalized data rather than ambient resolver or filesystem types.
