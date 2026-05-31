Evidence-ID: native-dynamic-plans-h2-undeclared-output-oracle
Task-ID: H2
Artifact-Type: oracle-checkpoint
Covers: build.engine.dynamic.plans.declared.outputs, defaults.dynamic.derivations
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-05-30

Question: Should an output that contains bytes shaped like `mantle-plan-v1` but was not named in `dynamic_plan_outputs` be reported as a rejected dynamic plan or ignored by native dynamic discovery?

Inspected evidence:
- `openspec/changes/native-dynamic-plans/specs/build-engine/spec.md` requires the scanner to inspect only declared outputs.
- `openspec/changes/native-dynamic-plans/design.md` uses declaration to avoid accidental scheduling and hidden graph growth.
- User direction favors native Mantle semantics over compatibility heuristics.

Decision: Undeclared outputs are ignored by the native dynamic-plan scanner. They do not schedule units and do not create native dynamic-plan report rows. Structured rejection is reserved for outputs explicitly declared in `dynamic_plan_outputs`.

Owner: agent, based on the fail-closed declared-output design; user may override before implementation.

Next action: Keep spec, design, and tests aligned on ignored undeclared outputs and rejected declared-invalid outputs.
