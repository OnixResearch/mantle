Evidence-ID: native-dynamic-plans-h1-abi-policy-oracle
Task-ID: H1
Artifact-Type: oracle-checkpoint
Covers: build.engine.dynamic.plans.abi, build.engine.dynamic.plans.declared.outputs, build.engine.dynamic.plans.scheduler, build.engine.dynamic.plans.provenance, defaults.dynamic.derivations
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-05-30

Question: Should native dynamic derivations depend on Steel or Nix `.drv` semantics, and what concrete ABI/policy choices unblock implementation?

Inspected evidence:
- User direction in this session: “okay no steel now.”
- Existing compatibility implementation: `crates/crunch-build/src/dynamic.rs` parses `.drv` ATerm outputs.
- Existing scheduler support: `crates/crunch-build/src/worker.rs` can call `want()` mid-run and register dynamic goals.
- Existing architecture boundary: `adr/0010-keep-mantle-build-tool-boundary.md` keeps Mantle build-shaped and frontend-neutral.

Decision: Use Rust-validated `mantle-plan-v1` canonical JSON as the native ABI. Exclude Steel and Nix IFD. Keep `.drv` detection as labeled compatibility behavior. Require exact `dynamic_plan_outputs` declarations. Include v1 declared source inputs as top-level store-prefix paths with optional BLAKE3 NAR digests, referenced by unit inputs. For v1, dynamic units inherit producer sandbox/trust/store-prefix policy and any widening syntax is rejected.

Owner: agent, based on explicit user direction; user may override before implementation.

Next action: Implement the OpenSpec tasks in order, starting with pure dynamic-plan ABI validation and digest tests.
