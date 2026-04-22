Evidence-ID: no-std-functional-core-v2-attestation-shell-boundary
Task-ID: V2
Artifact-Type: verification-note
Covers: architecture.nostd.core.crate.boundary.effectful.dependency.outside, functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads, functional.core.shell.adapters.effect.translation.attestation.file.discovery.in.shell
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-22

Validation command:
- `cargo test -p crunch-attestation shell_adapter_keeps_discovery_outside_core`
- Output: `test discovery::tests::shell_adapter_keeps_discovery_outside_core ... ok`
- Output: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s`

Code inspection confirmed:
- `crates/crunch-attestation/src/discovery.rs` still owns directory walking, `std::fs::read_dir`, file reads, and `Path`/`PathBuf` handling
- `crates/crunch-attestation/src/adapter.rs` translates borrowed/std-facing inputs into owned `crunch_attestation_core::*Input` structs before calling core functions
- legacy std files `src/{canonical,digest,error,policy,release,schema,version}.rs` remain adapter-only re-export surfaces

Result: attestation file discovery remains in the std shell, and `crunch-attestation-core` receives only normalized owned attestation data rather than performing ambient reads itself.
