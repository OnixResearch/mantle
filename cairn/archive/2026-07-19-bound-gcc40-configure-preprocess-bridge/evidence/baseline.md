# Baseline and approach registry

## Success contract

Goal: confine the provisional GCC 4.0 compile-backed preprocessing bridge to explicit bounded configure probes without promoting it to compiler or provider evidence.

Observable completion: runtime rejection precedes compiler execution for every out-of-scope fact; accepted invocations are bounded and audited; static source/evidence drift fails; positive and negative fixtures pass; broad gates either pass or identify an exact unrelated blocker.

False completions: comments alone, metadata alone, unrestricted `-E`, output produced before rejection, fallback after rejection, arbitrary source paths, unbounded counts/bytes, and any provider/compiler-correctness claim.

Budgets: repository and local deterministic evidence only; three mechanism families; one advisory review; two implementation/audit rounds before reassessment. Allowed outcomes: validated confinement, exact blocker, exhausted, or user-decision-required.

## Registry

| Family | Mechanism | Claim | Evidence | Gap | State |
|---|---|---|---|---|---|
| native-replacement | Replace the branch with a conforming source-built preprocessor | Removes bridge | Existing logs record TinyCC `-E` sentinel failure and later native GCC remains a broader correctness frontier | Stronger | Blocked |
| configure-cache | Preseed every needed Autoconf result and remove `-E` execution | Avoids bridge execution | No complete reviewed cache or equivalence proof exists; hidden cache facts could exceed the original gap | Equivalent/unknown | Falsified |
| runtime-confinement | Admit only canonical bounded `conftest.c` requests under explicit per-configure authority, then audit | Proves scope confinement, not preprocessing correctness | Exact wrapper/configure sites and accepted `gcc40-bridge` non-claim support this narrower result | Simpler | Active |

## Advisory audit

A secondary model suggested filename and source-tree restrictions but also proposed unbounded ordinary-source behavior that would contradict the goal. That advice was rejected. The surviving route requires explicit authority, canonical containment, no output, named byte/count limits, exact class admission, pre-execution audit, and post-configure audit verification.