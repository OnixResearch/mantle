## ADDED Requirements

### Requirement: GCC 4.0 Correctness Promotion Plan [r[gcc40-correctness-roadmap]]
Crunch MUST track GCC 4.0 correctness promotion separately from pass1 graph completion, with verifiable milestones for replacing stubs and adding semantic smokes.

#### Scenario: Graph completion caveat remains explicit [r[gcc40-correctness-roadmap.1]]
- GIVEN the GCC 4.0 artifact builds successfully
- WHEN the correctness promotion plan is reviewed
- THEN it distinguishes bridge graph-completion from native/self-hosted/correct GCC behavior

#### Scenario: Milestones are independently verifiable [r[gcc40-correctness-roadmap.2]]
- GIVEN a future implementation task is selected
- WHEN its verification is run
- THEN the task proves a specific semantic or executable behavior rather than broad completion
