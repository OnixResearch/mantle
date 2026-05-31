Evidence-ID: native-dynamic-plans-v2-worker-integration
Task-ID: V2
Artifact-Type: planned-verification
Covers: build.engine.dynamic.plans.declared.outputs, build.engine.dynamic.plans.scheduler, defaults.dynamic.derivations
Reviewer-Role: agent
Verdict: planned
Reviewed-At: 2026-05-30

Planned evidence: worker integration tests for valid native plan scheduling, root-only scheduling, root dependency-closure builds, unreachable non-root units registered but unbuilt, undeclared-output ignore behavior, invalid declared-plan rejection, policy-widening rejection, and compatibility `.drv` separation after I6-I8 implementation.
