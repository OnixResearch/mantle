## ADDED Requirements

### Requirement: Project file generation is explicit and reviewable [r[project_workflows.project_filegen]]

Mantle MUST support project-declared generated files through an explicit no-mutate planning workflow and an explicit apply workflow. File generation declarations MUST name target paths, content sources, materialization methods, and content identity, and Mantle MUST reject path escapes, unsupported methods, conflicts, and plan drift before mutation.

#### Scenario: Filegen plan is side-effect free [r[project_workflows.project_filegen.scenario.plan]]

- GIVEN a project declares generated files with target paths, content sources, and materialization methods
- WHEN an operator runs the file generation plan command
- THEN Mantle MUST report deterministic create, update, unchanged, stale, and conflict operations without writing files, lockfiles, generated inputs, or store state
- AND the plan MUST include enough content identity for review before apply.

#### Scenario: Filegen apply writes only accepted operations [r[project_workflows.project_filegen.scenario.apply]]

- GIVEN a generated-file plan has no blocking conflicts and the operator explicitly applies it
- WHEN Mantle materializes generated files
- THEN Mantle MUST write only the files and materialization methods named by the verified plan
- AND it MUST fail before writing if current-file facts differ from the reviewed plan.

#### Scenario: Target escapes and conflicts fail closed [r[project_workflows.project_filegen.scenario.conflict]]

- GIVEN a generated file target is absolute, normalizes above the project root, collides with an unmanaged file, or uses an unsupported materialization method
- WHEN Mantle plans or applies file generation
- THEN Mantle MUST emit a deterministic blocker naming the unsafe target or method
- AND it MUST NOT create, replace, symlink, or delete that target.

### Requirement: Generated project files may be typed content [r[project_workflows.project_filegen_typed_content]]

Mantle MUST allow generated file declarations to bind content to a Nickel contract or schema-derived contract when the project supplies one. Contract validation MUST happen before file materialization, and success MUST be reported as generated-content validation only, not as frontend deployability or build success.

#### Scenario: Valid generated content is materializable [r[project_workflows.project_filegen_typed_content.scenario.valid]]

- GIVEN a generated file declaration includes content plus a contract binding
- WHEN Mantle validates the generated content before apply
- THEN Mantle MUST accept the content only if it satisfies the declared contract
- AND the filegen evidence MUST bind the contract identity and generated content digest.

#### Scenario: Invalid generated content blocks apply [r[project_workflows.project_filegen_typed_content.scenario.invalid]]

- GIVEN generated content fails its declared contract or schema-derived contract
- WHEN Mantle plans or applies file generation
- THEN Mantle MUST report a deterministic contract-validation diagnostic
- AND it MUST NOT materialize the invalid generated file.

#### Scenario: Typed generated file claim is bounded [r[project_workflows.project_filegen_typed_content.scenario.non-claim]]

- GIVEN generated content satisfies its declared contract and is materialized
- WHEN Mantle renders human output, JSON output, docs, or evidence
- THEN the claim MAY state that the generated file matches the declared content digest and contract identity
- AND it MUST NOT claim deployability, service readiness, frontend module correctness, or build success without separate evidence.
