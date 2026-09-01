# Build Correctness Specification

## Purpose

Defines the `build-correctness` capability.

## Requirements

### Requirement: Mantle action specs are declared build actions [r[build_correctness.action_spec]]

Mantle MUST model strong build-correctness claims with a versioned `mantle-action-spec-v1` record. The action spec MUST declare action kind, platform, toolchain object refs, input object refs, argument digest, environment digest, output declarations, sandbox policy, network policy, expected reference policy, and frontend spec refs when present. The canonical action ref MUST use BLAKE3 with domain separation and the shape `mantle-action://blake3/<digest>`.

#### Scenario: Canonical action ref is deterministic [r[build_correctness.action_spec.scenario.deterministic]]

- GIVEN two action specs contain equivalent declared fields in different input or map traversal orders
- WHEN Mantle canonicalizes them
- THEN both specs MUST produce the same action ref
- AND host temp paths, serialization order, and undeclared environment MUST NOT affect the action ref

#### Scenario: Changed declared input changes action ref [r[build_correctness.action_spec.scenario.semantic-drift]]

- GIVEN an action changes toolchain ref, input object ref, args digest, environment digest, output declaration, sandbox policy, network policy, expected refs, or frontend spec ref
- WHEN Mantle canonicalizes the changed spec
- THEN it MUST produce a different action ref
- AND prior receipts for the old action ref MUST NOT satisfy the changed action

### Requirement: Mantle Nickel evaluation source closure [r[build_correctness.nickel_eval_source_closure]]

Mantle MUST model Nickel evaluation as a declared source-closure action whenever the evaluated result participates in a strong build-correctness claim. The evaluation receipt MUST bind root source ref, transitive dependency refs or import closure, import path policy, evaluator identity, selected export format or build-IR shape, and output digest. Undeclared imports, ambient filesystem reads, evaluator mismatch, stale dependency refs, or output digest mismatch MUST fail closed for strong correctness claims.

#### Scenario: Declared Nickel eval is receipt-bound [r[build_correctness.nickel_eval_source_closure.scenario.bound]]

- GIVEN a Mantle `.ncl` build expression has a declared root source, dependency closure, evaluator identity, import path policy, and output format
- WHEN Nickel evaluation produces build input data
- THEN Mantle MUST emit an evaluation receipt binding those fields and the output digest
- AND downstream action specs MAY reference that evaluation receipt as provenance

#### Scenario: Undeclared Nickel import blocks strong claim [r[build_correctness.nickel_eval_source_closure.scenario.undeclared-import]]

- GIVEN a Mantle `.ncl` expression imports a file outside the declared source closure or allowed import path policy
- WHEN Nickel evaluation provenance is required for a strong correctness claim
- THEN Mantle MUST fail closed with a deterministic undeclared-import diagnostic
- AND it MUST NOT treat the lowered build data as strong correctness evidence

### Requirement: Mantle CAS object store [r[build_correctness.cas_object_store]]

Mantle MUST represent admitted build inputs and produced outputs as content-addressed objects using explicit BLAKE3 object refs such as `mantle-object://blake3/<digest>`. Object manifests MUST model object kind, byte count when applicable, content digest, executable or mode metadata when modeled, symlink target when applicable, sorted directory children when applicable, and redacted secret descriptor metadata. Path roots MAY be recorded as views, but path roots MUST NOT be accepted as canonical object identity.

#### Scenario: Produced object is admitted by content [r[build_correctness.cas_object_store.scenario.admit]]

- GIVEN a produced output has modeled metadata and bytes
- WHEN CAS admission runs
- THEN Mantle MUST compute a canonical object ref from content and modeled metadata
- AND any export or execution path MUST be recorded only as a view over that object ref

#### Scenario: Path-only identity is rejected [r[build_correctness.cas_object_store.scenario.reject-path-only]]

- GIVEN a build input or output is identified only by a host path, export path, or execution path
- WHEN strong correctness admission runs
- THEN Mantle MUST reject the object or mark the claim as unsupported
- AND diagnostics MUST identify path-only identity as the blocker

### Requirement: Hermetic execution policy [r[build_correctness.hermetic_execution_policy]]

Mantle MUST require an explicit sandbox and network policy before reporting strong action-correct execution. The execution report MUST state whether the requested policy was enforced. If Mantle cannot enforce the requested policy on the current host, it MUST fail closed or downgrade the evidence to a narrower non-strong claim.

#### Scenario: Enforced policy enables strong receipt [r[build_correctness.hermetic_execution_policy.scenario.enforced]]

- GIVEN an action spec declares sandbox and network policy supported by the current executor
- WHEN Mantle executes the action
- THEN the action receipt MUST include a sandbox report proving the requested policy was enforced
- AND the result MAY participate in strong action-correctness claims

#### Scenario: Unsupported policy blocks strong claim [r[build_correctness.hermetic_execution_policy.scenario.unsupported]]

- GIVEN an action spec requires sandbox or network restrictions the current executor cannot enforce
- WHEN Mantle prepares execution or receipt admission
- THEN Mantle MUST fail closed or mark the result fixture-only/narrower evidence
- AND it MUST NOT emit a strong action-correct receipt

### Requirement: Output reference scanning [r[build_correctness.output_reference_scanning]]

Mantle MUST provide output reference scan evidence for strong build-correctness claims. The scan MUST compare discovered or supplied output references against the action spec's expected reference policy. Undeclared refs, forbidden refs, duplicate conflicting views, path traversal, stale scan roots, unsupported scanner kinds, and plaintext secret bytes MUST fail closed.

#### Scenario: Declared references pass [r[build_correctness.output_reference_scanning.scenario.pass]]

- GIVEN a produced output references only declared input objects, generated payloads, entrypoints, and redacted secret descriptors
- WHEN reference scanning validates the output
- THEN the scan report MUST be accepted
- AND the action receipt MUST bind the accepted scan report

#### Scenario: Forbidden references fail [r[build_correctness.output_reference_scanning.scenario.fail]]

- GIVEN a produced output references an undeclared host path, temp/build root, frontend-forbidden runtime path, path traversal, or plaintext secret bytes
- WHEN reference scanning validates the output
- THEN the scan report MUST fail with deterministic diagnostics
- AND the action result MUST NOT satisfy strong correctness admission

### Requirement: Reuse and substitution admission [r[build_correctness.reuse_and_substitution]]

Mantle MUST accept reused or substituted outputs for strong correctness claims only when the candidate receipt matches the requested action ref, input refs, toolchain refs, sandbox policy, network policy, output object refs, reference scan policy, producer policy, and required signatures. Stale refs, missing signatures when required, policy mismatch, unsupported producer identity, path-only identity, or receipt tampering MUST fail closed.

#### Scenario: Matching receipt admits reuse [r[build_correctness.reuse_and_substitution.scenario.accept]]

- GIVEN a prior local or external receipt matches the requested action ref and required trust policy
- WHEN Mantle evaluates output reuse
- THEN it MAY accept the produced object refs without rerunning the action
- AND the new report MUST explain reuse with the matched receipt identity

#### Scenario: Stale substitute is rejected [r[build_correctness.reuse_and_substitution.scenario.reject-stale]]

- GIVEN a candidate substitute has a stale action ref, stale object ref, mismatched sandbox policy, missing required signature, unsupported producer identity, or path-only output identity
- WHEN Mantle evaluates output reuse
- THEN it MUST reject the substitute with deterministic diagnostics
- AND it MUST NOT report strong action-correct success for that output

### Requirement: Bootstrap toolchain outputs are deterministic [r[build_correctness.bootstrap_toolchain_determinism]]

Mantle bootstrap derivations that participate in self-hosting or release-witness correctness claims MUST produce deterministic content-addressed outputs for equivalent declared inputs, build scripts, sandbox policy, and bootstrap seed material. Timestamp generation, archive member metadata, locale-sensitive ordering, umask, temporary paths, hostnames, user names, and host tool discovery order MUST either be fixed to deterministic values or excluded from the admitted output identity.

#### Scenario: equivalent GCC bootstrap builds converge

GIVEN two fresh Mantle stores build `bootstrap/gcc.ncl` from equivalent declared seed, source, binutils, musl, make, dash, GMP, MPFR, and MPC inputs
WHEN both builds complete under the same declared sandbox and network policy
THEN the resulting GCC output object digest and content-addressed store path MUST match
AND the build report MUST record the deterministic environment policy used for time, locale, umask, archive behavior, and install metadata.

#### Scenario: nondeterministic bootstrap output blocks strong self-hosting claims

GIVEN two bootstrap toolchain builds from equivalent declared inputs produce different content-addressed output paths or object digests
WHEN Mantle evaluates a self-hosting or release-witness correctness claim that depends on that toolchain
THEN Mantle MUST report the first divergent bootstrap output with deterministic diagnostics
AND it MUST NOT claim cross-machine self-hosting or release-witness reproducibility for downstream binaries built with the divergent toolchain.

#### Scenario: bootstrap path aliases do not weaken tool identity

GIVEN a self-build script maps admitted bootstrap outputs to stable in-sandbox aliases for compiler, linker, archiver, Rust, busybox, or bwrap paths
WHEN Mantle records the build receipt or proof manifest
THEN the receipt MUST bind the real tool object refs and real content-addressed output paths separately from the stable execution aliases
AND stable aliases MUST NOT be accepted as a substitute for matching tool object refs.

### Requirement: Mantle provides declared Nickel export actions [r[build_correctness.nickel_export_action]]

Mantle MUST provide a bounded Nickel export action that evaluates declared Nickel sources to a requested external format and emits a deterministic evaluation receipt. The receipt MUST bind root source refs, dependency refs, import-path policy, evaluator identity, selected format, output digest, and bounded non-claims.

#### Scenario: Declared export produces receipt-bound output [r[build_correctness.nickel_export_action.scenario.receipt]]

- GIVEN a Nickel export request declares source files, dependency files, import paths, output format, output target policy, and evaluator identity
- WHEN Mantle evaluates the request
- THEN the export receipt MUST bind the declared source closure, import policy, evaluator identity, format, and output digest
- AND downstream action specs MAY cite that receipt as Nickel evaluation provenance.

#### Scenario: Unsafe import path fails closed [r[build_correctness.nickel_export_action.scenario.import-escape]]

- GIVEN an export request includes an absolute import path or an import path that normalizes above the declared root
- WHEN Mantle validates the request
- THEN Mantle MUST reject the request before invoking the Nickel evaluator
- AND it MUST report a deterministic import-path diagnostic without reading escaped files.

#### Scenario: Undeclared dependency blocks strong export claim [r[build_correctness.nickel_export_action.scenario.undeclared-dependency]]

- GIVEN a Nickel export depends on a source file outside the declared source and dependency closure
- WHEN Mantle requires export provenance for a strong correctness claim
- THEN Mantle MUST fail closed with an undeclared-dependency diagnostic
- AND it MUST NOT emit a receipt that implies the output is fully declared.

### Requirement: Mantle records Nickel evaluator toolchain facts [r[build_correctness.nickel_toolchain_provider]]

Mantle MUST model the Nickel evaluator used by export and build-evaluation paths as explicit toolchain data. The descriptor MUST include evaluator binary identity, version, and evaluator options, and MUST be recorded separately from frontend or Bazel-specific toolchain mechanisms.

#### Scenario: Evaluator descriptor participates in identity [r[build_correctness.nickel_toolchain_provider.scenario.identity]]

- GIVEN two Nickel export requests differ only by evaluator binary identity, version, or evaluator options
- WHEN Mantle computes their evaluation receipt identity
- THEN the receipts MUST differ
- AND a receipt from one evaluator descriptor MUST NOT satisfy a request using the other descriptor.

#### Scenario: Toolchain descriptor remains frontend neutral [r[build_correctness.nickel_toolchain_provider.scenario.frontend-neutral]]

- GIVEN an external frontend or adapter supplies a Nickel evaluator descriptor
- WHEN Mantle records the descriptor in export evidence
- THEN Mantle MUST treat binary identity, version, and options as data
- AND it MUST NOT require Bazel repository rules, Nix flakes, Onix module semantics, or other frontend-specific toolchain machinery in core.

### Requirement: Host-tool attestation inventory

r[build_correctness.host_tool_attestation_inventory] Mantle MUST require any host executable used by strict proof or sandbox setup paths to be declared in a host-tool inventory with role, absolute path, BLAKE3 digest, bounded version evidence, and provenance before that executable can affect proof evidence.

#### Scenario: declared host tool is accepted

GIVEN a host-tool inventory declares an executable role, absolute path, BLAKE3 digest, bounded version evidence, and provenance note
WHEN Mantle validates the inventory and observes that executable in a strict proof path
THEN the observation MUST match the accepted inventory record
AND proof reports MUST bind the inventory digest and accepted tool role.

#### Scenario: undeclared execution is denied

GIVEN protected execution observes an executable path with no accepted inventory record or Mantle-built sandbox transition record
WHEN the strict proof path evaluates the observation
THEN Mantle MUST deny execution or fail the proof before accepting evidence
AND the diagnostic MUST identify the undeclared executable class.

#### Scenario: digest drift blocks proof

GIVEN an inventory record names a host executable but its current bytes, path kind, or bounded version evidence no longer match
WHEN Mantle validates the inventory before strict proof execution
THEN validation MUST fail closed with a deterministic inventory-drift diagnostic
AND Mantle MUST NOT fall back to name-only or path-only trust.

### Requirement: No network during ordinary builds by default

r[build_correctness.no_network_by_default] Mantle MUST disable network access for ordinary derivation builds by default, keep network source acquisition inside declared fixed-output fetcher actions, and model any build-time network exception as an explicit audited sandbox capability.

#### Scenario: ordinary builder network attempt is blocked

GIVEN a normal derivation build has no declared network capability
WHEN the builder attempts to contact the network during sandbox execution
THEN Mantle MUST block the attempt or fail the build with a deterministic network-policy diagnostic
AND the output MUST NOT satisfy strong build-correctness evidence.

#### Scenario: fixed-output fetcher declares network input

GIVEN a builtin fetcher action declares URL, hash algorithm, expected digest, mode, and retry policy
WHEN Mantle acquires the source through that fetcher
THEN network access MAY occur only inside the fixed-output fetcher boundary
AND the admitted source MUST match the declared content hash before downstream builds use it.

#### Scenario: compatibility exception is scoped

GIVEN a foreign or compatibility build requests build-time network access
WHEN Mantle validates sandbox capability policy
THEN the exception MUST name the affected action, capability, policy basis, and audit class
AND an undeclared or policy-denied exception MUST fail closed before strong evidence is emitted.

### Requirement: Explicit build environment allowlist

r[build_correctness.explicit_environment_allowlist] Mantle MUST construct strict build environments from declared environment entries and reviewed deterministic defaults, reject denied ambient variables, and bind the normalized environment digest into action and build evidence.

#### Scenario: declared environment is stable

GIVEN a strict build action declares environment entries and deterministic defaults
WHEN Mantle normalizes the build environment
THEN the child environment MUST contain only declared or policy-default entries
AND the action or build receipt MUST bind the normalized environment digest.

#### Scenario: ambient poisoning is rejected

GIVEN the parent process contains denied ambient variables such as dynamic-linker controls, compiler wrappers, proxy settings, token-like names, or undeclared locale overrides
WHEN Mantle prepares a strict build environment
THEN Mantle MUST reject or omit those variables before child execution according to policy
AND diagnostics MUST identify the denied class without copying secret values into reports.

#### Scenario: missing required variable fails closed

GIVEN a strict action requires a modeled environment entry that cannot be constructed from declared inputs or deterministic defaults
WHEN environment normalization runs
THEN Mantle MUST fail before execution with a deterministic missing-environment diagnostic
AND it MUST NOT inherit the missing value from the host environment.

### Requirement: Receipt-bound executable search path

r[build_correctness.receipt_bound_path] Mantle MUST derive strict build executable search paths from declared tool references or accepted host-tool inventory records, and receipts MUST bind the ordered search path, alias views, and real tool identities used for execution.

#### Scenario: declared tool path is accepted

GIVEN a strict build action declares tool references needed by the builder
WHEN Mantle constructs the build search path
THEN every PATH entry MUST map to a declared tool ref or accepted host-tool inventory record
AND the build receipt MUST bind the ordered path digest and real tool identities.

#### Scenario: ambient path poisoning fails closed

GIVEN the parent PATH contains an undeclared directory or executable that would shadow a declared tool
WHEN Mantle prepares a strict build
THEN Mantle MUST reject the unclassified PATH influence before execution
AND it MUST NOT search the ambient PATH to repair missing declared tools.

#### Scenario: alias identity remains explicit

GIVEN Mantle materializes stable executable aliases inside a sandbox
WHEN a build or proof report records executable authority
THEN the report MUST distinguish alias paths from the real content-addressed tool refs or host-tool inventory records
AND alias names MUST NOT satisfy tool identity without matching real refs.

### Requirement: Determinism normalization policy

r[build_correctness.determinism_normalization_policy] Mantle MUST bind a strict determinism normalization policy for proof-grade builds, covering time, timezone, locale, umask, temp roots, host/user metadata, modeled randomness, and order-sensitive output processing.

#### Scenario: equivalent builds converge

GIVEN two strict builds use equivalent declared inputs and the same determinism normalization policy
AND ambient host time, locale, umask, temp roots, user names, and environment noise differ
WHEN Mantle admits build evidence for those outputs
THEN the admitted output digests and relevant receipt fields MUST match or the divergence MUST be reported deterministically
AND the receipt MUST bind the normalization policy used for the comparison.

#### Scenario: unsupported normalization blocks strong claims

GIVEN a requested strict build or proof requires a normalization control that the current executor cannot enforce
WHEN Mantle evaluates strong build-correctness, self-hosting, release, or reproducibility eligibility
THEN Mantle MUST mark the affected claim as blocked or unsupported
AND it MUST NOT silently continue with a partial normalization policy.

#### Scenario: nondeterministic divergence is diagnostic evidence only

GIVEN two proof-grade runs from equivalent declared inputs produce different content-addressed output digests
WHEN Mantle summarizes the proof result
THEN the report MUST identify the first modeled divergent surface when available
AND it MUST NOT claim deterministic release, global reproducibility, or strong cross-run equivalence for the divergent artifact.

### Requirement: Release determinism uses a genuine rebuild

r[mantle.build_correctness.release_determinism.genuine_rebuild] Mantle MUST emit a promoting deterministic release proof only when every proof run executes a reviewed rebuild recipe over declared source and toolchain inputs without read authority over the published target artifacts, content-identical aliases, prior proof outputs, or the ordinary reproduce output.

#### Scenario: Declared source rebuilds converge

r[mantle.build_correctness.release_determinism.fixtures.positive]
- GIVEN two proof runs receive the same declared source, recipe, toolchain, provider, sandbox, and normalization identities through distinct fresh stores
- AND neither run can read any published target identity
- WHEN both runs produce the exact selected artifact set with matching BLAKE3 digests
- THEN Mantle MAY classify the bounded result as a genuine `self-rebuild-match` contribution.

#### Scenario: Copying the target cannot promote

r[mantle.build_correctness.release_determinism.fixtures.negative.target_copy]
- GIVEN a rebuild helper attempts to copy the published binary directly or through a content-identical alternate path, symlink, hardlink, proof-bundle entry, or prior output
- WHEN Mantle plans or executes a deterministic proof run
- THEN the target bytes MUST be unavailable to the rebuild process
- AND the proof MUST fail with a deterministic target-authority or undeclared-input blocker.

### Requirement: Rebuild identity is content-bound

r[mantle.build_correctness.release_determinism.identity_binding] Mantle MUST bind canonical recipe bytes, executable and tool byte identities, ordered arguments, source and input closure identities, provider and target identities, sandbox policy, effect policy, normalization policy, and approved read/write roots into a versioned BLAKE3 rebuild descriptor cited by every proof run.

#### Scenario: Recipe or tool drift fails closed

r[mantle.build_correctness.release_determinism.fixtures.negative.identity_drift]
- GIVEN a recipe, executable, tool, source closure, argument, provider, or policy changes while a path or label remains unchanged
- WHEN a proof run is compared with its accepted rebuild descriptor
- THEN the content-bound identity MUST differ or validation MUST fail
- AND the stale descriptor MUST NOT contribute to a promoting verdict.

### Requirement: Rebuild authority planning is pure

r[mantle.build_correctness.release_determinism.authority_plan] Mantle MUST decide allowed rebuild inputs, target-identity exclusions, run-root separation, and deterministic proof eligibility in a pure core over normalized observations, while byte measurement, capability-root setup, sandbox execution, output comparison, and receipt writing remain in the shell.

#### Scenario: Authority plan is testable without execution

r[mantle.build_correctness.release_determinism.authority_plan.test]
- GIVEN in-memory published target identities, candidate input observations, run roots, and policy
- WHEN the authority planner evaluates them
- THEN it MUST deterministically return an allowed plan or ordered blockers without reading files, environment state, clocks, networks, or processes.

### Requirement: Shared action results are immutable content-bound records
r[build_correctness.shared_action_result_records]

Mantle MUST represent a publishable action result as a versioned immutable record binding the requested action ref, ordered output declarations and object refs, PathInfo refs, action receipt ref, required reference-scan and sandbox/network-policy evidence, producer identity and policy, signatures, publication-policy identity, and bounded non-claims. Mantle-owned record identity MUST use domain-separated BLAKE3. A local derivation-to-output mapping, mutable index row, object-presence fact, or execution success alone MUST NOT constitute an action-result record.

#### Scenario: Admitted result becomes publishable

- GIVEN an action completed and all declared outputs, objects, PathInfo, receipts, scans, policies, and required signatures passed ordinary admission
- WHEN Mantle constructs the action-result record
- THEN it MUST bind those canonical facts to one immutable record ref
- AND the record MAY be published only after every referenced required artifact is durable.

#### Scenario: Incomplete local mapping is not promoted

- GIVEN a local CA derivation mapping names an output path but lacks a complete admitted action receipt, object closure, required signatures, or policy evidence
- WHEN Mantle evaluates it for shared publication
- THEN Mantle MUST reject or retain it as a local advisory hint
- AND it MUST NOT publish or report it as a shared admitted action result.

#### Scenario: Record identity is deterministic

- GIVEN two result records contain equivalent canonical facts in different map, output, signature, or reference traversal orders
- WHEN Mantle canonicalizes them
- THEN both MUST produce the same action-result ref
- AND host paths, publication time, transport order, and mutable index position MUST NOT affect that ref.

### Requirement: Shared action-result admission fails closed
r[build_correctness.shared_action_result_admission]

Mantle MUST treat shared action-result lookup as advisory discovery and MUST admit a candidate only after validating its action ref, output declarations and object refs, object completeness, PathInfo, receipt linkage, signatures, producer policy, sandbox/network policy, reference-scan policy, and requested claim strength. If multiple otherwise admissible candidates for one action ref name differing output object sets, strong reuse MUST fail with bounded conflict evidence rather than select by arrival, source order, or last writer.

#### Scenario: Matching shared result avoids execution

- GIVEN a discovered candidate matches the requested action and every required trust, policy, object, receipt, and scan fact
- WHEN Mantle plans reuse
- THEN it MAY admit the candidate outputs without rerunning the action
- AND the report MUST identify the selected result ref, trust basis, and discovery source.

#### Scenario: Poisoned candidate is rejected

- GIVEN a candidate has a stale action ref, altered output ref, incomplete object tree, missing or invalid signature, unsupported producer, mismatched policy, malformed receipt linkage, or path-only identity
- WHEN Mantle evaluates reuse
- THEN it MUST reject the candidate with deterministic diagnostics
- AND it MUST NOT mutate admitted store state or report a cache hit from that candidate.

#### Scenario: Conflicting admitted results expose nondeterminism

- GIVEN two candidates for the same action ref each pass individual shape and trust checks but name different output object sets
- WHEN strong reuse admission compares the candidate set
- THEN Mantle MUST reject automatic strong reuse with `conflicting-action-results`
- AND bounded evidence MUST identify candidate refs and output-set digests without selecting by source order.

### Requirement: Stateful workspace execution modes are explicit
r[build_correctness.stateful_workspace_modes]

Mantle MUST require an explicit workspace mode when retained tool state is available. `none` MUST use no retained workspace state; `immutable-snapshot` MUST use declared read-only content-addressed snapshot objects that participate in action identity; and `mutable-session` MUST use a bounded leased writable workspace whose compatibility and authority are validated before sandbox start. Mantle MUST NOT silently fall back from one mode to another.

#### Scenario: Immutable snapshot is a declared input

- GIVEN an action declares a compatible immutable workspace snapshot by canonical object ref and stable guest mount path
- WHEN Mantle constructs and executes the action
- THEN the snapshot ref and mount declaration MUST participate in action identity and ordinary input admission
- AND host storage paths or prior mutable workspace identity MUST NOT affect that identity.

#### Scenario: Mutable workspace requires compatible lease

- GIVEN an action requests mutable-session mode
- WHEN Mantle validates the workspace
- THEN worker, authority class, action compatibility, toolchain refs, guest path, current job/attempt/fence, and quota policy MUST match
- AND any mismatch MUST reject workspace reuse before sandbox execution.

#### Scenario: Mode fallback is not implicit

- GIVEN the requested snapshot is missing or the mutable workspace is unavailable, quarantined, stale, or over quota
- WHEN Mantle plans execution
- THEN it MUST fail or choose another mode only under an explicit configured fallback
- AND the report MUST identify the actual mode and fallback reason.

### Requirement: Mutable workspace execution has a narrower claim boundary
r[build_correctness.mutable_workspace_claim_boundary]

Mantle MUST classify mutable-session workspace content as execution history rather than a declared immutable action input. An execution that reads mutable workspace state MUST NOT by itself publish or satisfy a strong shared action result. A separate clean execution from equivalent declared inputs MAY provide comparison evidence when its admitted output object set matches, but it MUST NOT retroactively relabel the original mutable execution as hermetic.

#### Scenario: Warm build reports narrower evidence

- GIVEN a build reads a compatible mutable leased workspace and produces outputs that pass ordinary content and output admission
- WHEN Mantle reports the result
- THEN it MAY report successful practical execution and admitted output objects
- AND it MUST state that mutable workspace history prevents strong hermetic shared-reuse admission from that run alone.

#### Scenario: Mutable result is excluded from shared action cache

- GIVEN an action result was produced using mutable-session mode without accepted clean-rebuild comparison evidence
- WHEN Mantle considers shared action-result publication or strong reuse
- THEN it MUST reject that candidate with a stable mutable-state reason
- AND it MUST NOT treat matching output content alone as proof that workspace history was irrelevant.

#### Scenario: Clean comparison records equivalence narrowly

- GIVEN a separate `none` or declared immutable-snapshot rebuild uses equivalent declared action inputs and produces the same admitted output object set as a warm build
- WHEN Mantle evaluates comparison evidence
- THEN it MAY record output-set agreement bound to both executions
- AND it MUST NOT claim general tool-cache correctness, future determinism, or hermeticity of the original warm execution.

#### Scenario: Sensitive or escaping state is quarantined

- GIVEN a workspace contains policy-defined secret material, host-path leakage, path traversal, escaping symlinks, incompatible ownership, or content that cannot be scrubbed within bounds
- WHEN Mantle prepares reuse or snapshotting
- THEN it MUST reject and quarantine the workspace before another action reads it
- AND no shared snapshot or action result may reference the rejected state.

### Requirement: Advertised source-root capability is executable or explicitly unsupported
r[mantle.build_correctness.source_root_capability] Mantle MUST NOT advertise a source-root operation as executable when every invocation unconditionally returns unavailable. The operation MUST either execute a bounded receipt-producing implementation or be omitted from executable command discovery and reported as an explicit unsupported capability with a deterministic reason.

#### Scenario: Supported source-root operation executes
- GIVEN the host and explicit inputs satisfy the declared source-root capability contract
- WHEN the operator invokes the source-root operation
- THEN Mantle MUST execute the bounded operation and emit a receipt identifying inputs, policy, observed capability, outputs, and non-claims.

#### Scenario: Unsupported source-root capability is honest
- GIVEN the implementation or host cannot support source-root execution
- WHEN command discovery or capability reporting runs
- THEN Mantle MUST report the operation as unsupported before execution and MUST NOT expose a command path whose only outcome is an unconditional unavailable error.

### Requirement: Source-root logic preserves core and shell boundaries
r[mantle.build_correctness.source_root_capability.boundary] Mantle MUST keep source-root planning and capability decisions pure over supplied observations while filesystem discovery, host probing, source loading, process execution, and receipt writing remain in the shell.

#### Scenario: Pure planning is testable without host setup
- GIVEN source-root capability observations and a requested operation in memory
- WHEN planning runs
- THEN it MUST produce an execute or unsupported decision without reading files, environment state, clocks, network resources, or processes.

### Requirement: Onix release profiles require strict hermeticity
r[mantle.build_correctness.onix_release_strict_hermeticity] Mantle MUST require strict hermetic mode for Onix release evidence and MUST fail before pass evidence when declared sandbox, network, environment, path, clock, or tool restrictions cannot be enforced.

#### Scenario: Clean strict execution may pass
r[mantle.build_correctness.hermetic_handoff.fixtures.positive]
- GIVEN the executor enforces every strict restriction and records matching evidence
- WHEN the Onix release profile evaluates the build
- THEN the hermeticity contribution MAY pass.

#### Scenario: Host influence fails strict release evidence
r[mantle.build_correctness.hermetic_handoff.fixtures.negative]
- GIVEN execution observes undeclared host tools, ambient environment, network access, unstable paths, clock dependence, or unenforced sandbox policy
- WHEN the Onix release profile evaluates the build
- THEN the strict hermeticity contribution MUST fail.

#### Scenario: Practical mode cannot satisfy strict profile
- GIVEN a practical-mode build completes and emits diagnostic or development evidence
- WHEN an Onix release profile requires strict hermeticity
- THEN the practical receipt MUST NOT satisfy the strict evidence field or be promoted by summary metadata.

### Requirement: Build boundary remains bounded
r[mantle.build_correctness.hermetic_handoff.docs] Mantle documentation MUST distinguish executable source-root capability, explicit unsupported capability, strict release evidence, and practical diagnostic evidence.

#### Scenario: Strict evidence avoids overclaim
- GIVEN strict hermetic execution passes
- WHEN the result is documented
- THEN it MUST NOT claim compiler correctness, source correctness, semantic equivalence, universal reproducibility, deployment safety, or release eligibility outside the configured profile.

### Requirement: Dynamic-plan semantic values are nominal

r[build_correctness.dynamic_plan_nominal.values] Mantle MUST represent admitted unit IDs, source IDs, logical store paths, and output names with distinct private Rust types that enforce the existing scalar rules.

#### Scenario: Valid wire plan admits typed values

r[build_correctness.dynamic_plan_nominal.values.valid]
- GIVEN a `mantle-plan-v1` wire record contains bounded valid unit IDs, source IDs, logical store paths, and output names
- WHEN plan admission runs
- THEN Mantle MUST construct distinct admitted value types before graph validation.

#### Scenario: Invalid scalar fails before graph use

r[build_correctness.dynamic_plan_nominal.values.validation]
- GIVEN a plan contains an empty, oversized, control-bearing, malformed, or store-prefix-invalid semantic value
- WHEN plan admission runs
- THEN admission MUST fail with a deterministic scalar diagnostic
- AND graph validation MUST NOT receive the invalid value.

### Requirement: Dynamic-plan digest roles are nominal

r[build_correctness.dynamic_plan_nominal.digests] Mantle MUST represent canonical plan BLAKE3 identity and declared source NAR BLAKE3 identity with distinct Rust types.

#### Scenario: Plan and NAR digests do not compile interchangeably

r[build_correctness.dynamic_plan_nominal.compile_time]
- GIVEN plan and NAR digests use distinct marker-role instantiations
- WHEN source passes a NAR digest to an API that requires a plan digest
- THEN the source MUST fail compilation with a type mismatch.

### Requirement: Dynamic-plan wire and core models are separate

r[build_correctness.dynamic_plan_nominal.wire_boundary] Mantle MUST preserve the current `mantle-plan-v1` serialized shape in explicit wire DTOs and MUST convert admitted values through one pure wire-to-core boundary.

#### Scenario: Wire projection remains compatible

r[build_correctness.dynamic_plan_nominal.wire_boundary.compatible]
- GIVEN an admitted typed plan is projected for serialization
- WHEN Mantle emits `mantle-plan-v1`
- THEN field names, scalar spellings, enum tags, nullable fields, and collection shapes MUST match the accepted wire contract.

### Requirement: Dynamic-plan graph validation stays typed

r[build_correctness.dynamic_plan_nominal.graph] Mantle graph validation MUST retain typed unit, source, output, path, and digest values until diagnostics or wire projection require text.

#### Scenario: Unit and source IDs do not compile interchangeably

r[build_correctness.dynamic_plan_nominal.graph.ids]
- GIVEN `UnitId` and `SourceId` wrap the same scalar representation
- WHEN source uses `SourceId` for a unit edge
- THEN the source MUST fail compilation.

#### Scenario: Wrong-role placeholder fails

r[build_correctness.dynamic_plan_nominal.graph.validation]
- GIVEN a placeholder names a source where a unit output is required or names a unit where a source is required
- WHEN placeholder admission and graph validation run
- THEN Mantle MUST reject the placeholder with a deterministic role diagnostic.

### Requirement: Dynamic-plan canonical identity remains stable

r[build_correctness.dynamic_plan_nominal.compatibility] The nominal-type migration MUST preserve accepted canonical JSON bytes and plan BLAKE3 identities.

#### Scenario: Canonical plan bytes remain stable

r[build_correctness.dynamic_plan_nominal.compatibility.golden]
- GIVEN an accepted `mantle-plan-v1` fixture is processed before and after the migration
- WHEN canonical bytes and plan digests are compared
- THEN they MUST remain equal unless a separate versioned plan schema change approves the difference.

### Requirement: Octet checks the migrated dynamic-plan core

r[build_correctness.dynamic_plan_nominal.octet] After the policy becomes available, Mantle MUST declare its dynamic-plan domains to the reviewed Octet nominal-domain policy.

#### Scenario: Raw aliases do not return

r[build_correctness.dynamic_plan_nominal.octet.guard]
- GIVEN the dynamic-plan core has migrated to nominal types
- WHEN the Octet policy checks the configured scope
- THEN direct primitive aliases and raw declared domain values MUST fail the selected check.

### Requirement: Typed plan claims remain bounded

r[build_correctness.dynamic_plan_nominal.docs] Mantle documentation MUST state that typed plan values prove local category separation and scalar admission only.

#### Scenario: Boundary remains visible

r[build_correctness.dynamic_plan_nominal.final_checks]
- GIVEN a typed plan passes admission, graph checking, and canonicalization
- WHEN Mantle states the supported claim
- THEN it MUST NOT claim store presence, build success, sandbox enforcement, source trust, compiler correctness, or release eligibility.

### Requirement: Reviewed nominal dynamic-plan integration is lossless

r[build_correctness.dynamic_plan_nominal.integration] Mantle MUST integrate a recorded immutable nominal dynamic-plan source commit without deleting unrelated target work or weakening admitted type boundaries.

#### Scenario: Integration binds exact source objects

r[build_correctness.dynamic_plan_nominal.integration.source]

GIVEN integration evidence records the source commit, tree, parent, subject, and changed-file set
WHEN Mantle starts the integration
THEN the actual Git objects MUST match the recorded source objects
AND a mutable branch name MUST NOT replace the immutable source identity.

#### Scenario: Integration starts from a clean preserved target

r[build_correctness.dynamic_plan_nominal.integration.target]

GIVEN main contains work that is not present in the source parent
WHEN an operator selects the integration target
THEN that work MUST have a committed preserved state
AND source application MUST occur in a clean dedicated integration worktree.

#### Scenario: Conflict resolution preserves both semantic sides

r[build_correctness.dynamic_plan_nominal.integration.preservation]

GIVEN a source file overlaps newer target work
WHEN the integration resolves that overlap
THEN the result MUST preserve unrelated target behavior and all reviewed nominal-domain behavior
AND it MUST NOT add raw-string fallback paths into admitted graph logic.

#### Scenario: Resolved target passes positive and negative checks

r[build_correctness.dynamic_plan_nominal.integration.validation]

GIVEN the source commit has been applied and all conflicts are resolved
WHEN focused and broader validation runs on the resolved tree
THEN valid plans MUST retain their accepted behavior
AND invalid wire values and compile-time role substitutions MUST still fail.

#### Scenario: Negative evidence rejects weakened boundaries

r[build_correctness.dynamic_plan_nominal.integration.validation.negative]

GIVEN a resolution exposes a nominal field, adds unrestricted conversion, erases a digest role, or bypasses wire admission
WHEN negative checks inspect or compile the resolved source
THEN the integration MUST fail before lifecycle closure.

#### Scenario: Canonical identity remains compatible

r[build_correctness.dynamic_plan_nominal.integration.compatibility]

GIVEN the accepted canonical `mantle-plan-v1` fixture
WHEN the resolved integration serializes and hashes the admitted plan
THEN its JSON bytes and BLAKE3 plan digest MUST match the reviewed source evidence.

#### Scenario: Policy checks retain zero targeted findings

r[build_correctness.dynamic_plan_nominal.integration.policy]

GIVEN the resolved core uses the declared Mantle nominal domains
WHEN Octet runs with the reviewed denial policy
THEN targeted primitive aliases, raw domain values, and invariant bypasses MUST remain absent.

#### Scenario: Integration evidence identifies the resolved tree

r[build_correctness.dynamic_plan_nominal.integration.evidence]

GIVEN all required validation has completed
WHEN Mantle records integration evidence
THEN the receipt MUST bind source commit, target commit, resolved tree, conflict decisions, checks, and bounded blockers.

#### Scenario: Lifecycle closes before main moves

r[build_correctness.dynamic_plan_nominal.integration.lifecycle]

GIVEN the integration tasks and gates pass
WHEN the integration branch is ready for operator review
THEN Mantle MUST sync and archive the integration change before its final integration commit
AND it MUST NOT push or move main without explicit operator instruction.

### Requirement: Store roles use distinct capability values

r[build_correctness.store_capabilities.roles] Mantle MUST give build realization, output lookup, root retention, source admission, action-result exchange, and store administration distinct Rust capability values with only their required operations.

#### Scenario: Builder receives bounded store authority

- **GIVEN** Mantle constructs a builder for local or remote realization
- **WHEN** the builder receives its store dependencies
- **THEN** it MUST receive only build-store and fixed action-result operations
- **AND** garbage collection, source import, backend replacement, repair, and arbitrary root mutation MUST NOT be available through those values

#### Scenario: Pipeline retains root authority separately

- **GIVEN** pipeline orchestration needs output facts and selected root registration after a build
- **WHEN** it constructs the builder and post-build shell
- **THEN** output lookup and root registration MUST remain separate from builder-owned authority
- **AND** the builder MUST NOT gain arbitrary pin, unpin, or garbage-collection access

### Requirement: Raw writable store services remain confined

r[build_correctness.store_capabilities.raw_service_confinement] Mantle MUST keep writable blob, directory, PathInfo, publisher, and action-result backend objects private to `crunch-store` or shell-owned compatibility code.

#### Scenario: Build code performs a supported store operation

- **GIVEN** build code needs closure resolution, NAR calculation, castore transformation, cache lookup, substitution, or output persistence
- **WHEN** it requests that operation
- **THEN** it MUST use a named high-level capability method
- **AND** it MUST NOT receive a raw service object that permits unrelated mutation

#### Scenario: Raw service escape is introduced

- **GIVEN** a build or pipeline API returns a writable store trait object, broad callback, or generic service escape hatch
- **WHEN** compile-fail examples or source-policy guards run
- **THEN** validation MUST fail with a deterministic authority-boundary finding
- **AND** a test-only constructor MUST NOT disable the production boundary broadly

### Requirement: Capability migration preserves store behavior

r[build_correctness.store_capabilities.compatibility] Mantle MUST preserve accepted store formats, PathInfo and attestation facts, report schemas, output identities, cache behavior, substitution behavior, and root outcomes during the capability migration.

#### Scenario: Supported realization paths remain equivalent

- **GIVEN** accepted fixtures for local builds, cache hits, remote substitution, content-addressed outputs, action-result reuse, publication, and selected root retention
- **WHEN** the fixtures run before and after the capability migration
- **THEN** their accepted output identities and machine-visible facts MUST remain equal
- **AND** any intentional compatibility difference MUST require a separate versioned change

#### Scenario: Rejected output stays uncommitted

- **GIVEN** output hash, signature, identity, policy, or admission checks reject a candidate
- **WHEN** the restricted store capabilities apply the decision
- **THEN** no PathInfo, exported output, attestation, root, action result, or success report MUST be committed for that candidate
- **AND** restricted authority MUST NOT weaken the existing fail-closed path

### Requirement: Store capability claims remain local

r[build_correctness.store_capabilities.claim_boundary] Mantle MUST limit store-capability claims to Rust API reachability and preserved tested behavior.

#### Scenario: Capability tests pass

- **GIVEN** positive behavior tests and negative API guards pass
- **WHEN** Mantle reports the result
- **THEN** it MAY claim that selected Rust callers lack the excluded methods
- **AND** it MUST NOT claim host filesystem confinement, sandbox correctness, output correctness, cache trust, or release eligibility from the type split alone

### Requirement: Admitted trust-boundary values are nominal

r[build_correctness.nominal_boundaries.admission] Mantle MUST convert structural input into checked nominal values before pure core logic uses semantic identifiers, paths, digests, or bounded quantities.

#### Scenario: Valid structural input admits nominal values

- **GIVEN** a bounded wire record contains values that satisfy the accepted scalar rules
- **WHEN** semantic admission runs
- **THEN** Mantle MUST construct checked domain types before graph, policy, evidence, or execution planning uses those values
- **AND** the admitted core MUST retain those types until diagnostics or wire projection requires text

#### Scenario: Invalid values cannot bypass admission

- **GIVEN** structural input contains an empty, oversized, control-bearing, malformed, zero, overflowing, or otherwise invalid semantic value
- **WHEN** direct wire admission or deserialization runs
- **THEN** Mantle MUST reject the value with a deterministic diagnostic
- **AND** derived or custom deserialization MUST NOT bypass the constructor invariant
- **AND** the pure core MUST NOT receive the invalid value

### Requirement: Semantic identifier and reference roles remain distinct

r[build_correctness.nominal_boundaries.identities] Mantle MUST use distinct Rust types for identifiers and references whose accidental exchange can change graph, protocol, artifact, or evidence meaning.

#### Scenario: Unrelated identifiers cannot be exchanged

- **GIVEN** stage, artifact, tool, request, session, endpoint, requirement, and release identifiers share a string representation
- **WHEN** source passes one role to an API that requires another role
- **THEN** the source MUST fail compilation or require an explicit checked conversion
- **AND** unrestricted primitive conversion MUST NOT erase the role inside admitted core logic

#### Scenario: Graph references retain resolved roles

- **GIVEN** structural graph input names an existing declared node
- **WHEN** graph admission resolves that reference
- **THEN** the admitted reference MUST retain the resolved node role
- **AND** a missing node or wrong-role node MUST fail before graph decisions use it

### Requirement: BLAKE3 format and selected digest roles are checked

r[build_correctness.nominal_boundaries.digests] Mantle MUST validate Mantle-owned lowercase BLAKE3 text at admission and MUST keep selected same-format digest roles distinct where substitution can change evidence meaning.

#### Scenario: Valid BLAKE3 value enters its role

- **GIVEN** structural input contains a lowercase BLAKE3 value with the accepted length
- **WHEN** digest admission runs for the declared role
- **THEN** Mantle MUST construct the checked digest value
- **AND** later core logic MUST NOT repeat raw format checks for that value

#### Scenario: Malformed or wrong-role digest is rejected

- **GIVEN** a digest has the wrong length, case, alphabet, algorithm, or semantic role
- **WHEN** Mantle admits or passes that digest to a role-specific API
- **THEN** admission MUST reject malformed text
- **AND** source-level wrong-role substitution MUST fail compilation or require an explicit checked conversion

#### Scenario: Interoperability digest remains algorithm tagged

- **GIVEN** a protocol or external format requires a non-BLAKE3 digest
- **WHEN** Mantle admits that value
- **THEN** it MUST retain the required algorithm, checked digest value, and interoperability reason as one validated value
- **AND** it MUST NOT relabel that digest as a Mantle-owned BLAKE3 identity

### Requirement: Unit-bearing quantities and path authorities are explicit

r[build_correctness.nominal_boundaries.units_and_paths] Mantle MUST distinguish bounded quantities and path classes when equal primitive representations have different units, limits, ordering rules, or authority.

#### Scenario: Quantity uses its declared unit and bound

- **GIVEN** time, bytes, counts, offsets, indexes, or generations enter an admitted core
- **WHEN** Mantle constructs the value
- **THEN** the type or containing aggregate MUST identify its unit and accepted range
- **AND** zero, overflow, or a value above the named bound MUST fail when the contract forbids it

#### Scenario: Related quantities preserve their relationship

- **GIVEN** two same-unit values form a validity window, range, or ordered boundary
- **WHEN** Mantle admits the pair
- **THEN** one aggregate constructor MUST validate their relationship
- **AND** later core logic MUST NOT receive an inverted or otherwise invalid pair

#### Scenario: Path roles cannot silently cross authority boundaries

- **GIVEN** absolute executable, specification, repository-relative, bundle-relative, logical-store, provisional, and final paths share text representations
- **WHEN** a core operation receives one path class
- **THEN** it MUST accept only the required checked role
- **AND** the checked path MUST NOT be described as proof of filesystem presence, authorization, or safe I/O

### Requirement: Nominal migrations preserve accepted wire identity

r[build_correctness.nominal_boundaries.compatibility] Mantle MUST project admitted values through accepted wire contracts and MUST preserve canonical bytes and identity unless a separate versioned change approves a difference.

#### Scenario: Accepted wire projection remains stable

- **GIVEN** an accepted fixture passes before and after a nominal migration
- **WHEN** Mantle compares field names, scalar spellings, enum tags, nullable fields, collection shapes, canonical bytes, and BLAKE3 identities
- **THEN** those values MUST remain equal
- **AND** a Rust API migration MUST use an explicit compatibility adapter where supported callers still need the old shape

#### Scenario: Validation retains bounded diagnostics

- **GIVEN** one structural record contains several invalid semantic values
- **WHEN** the accepted boundary promises bounded multi-issue reporting
- **THEN** Mantle MUST retain the structural wire value long enough to report the allowed issue set
- **AND** fail-fast custom deserialization MUST NOT silently reduce that diagnostic contract

### Requirement: Nominal domains have regression guards

r[build_correctness.nominal_boundaries.guard] Mantle MUST add compile-time and policy guards for migrated domains so raw primitive aliases, unchecked constructors, or unrestricted deserialization cannot silently return.

#### Scenario: Primitive regression is introduced

- **GIVEN** a migrated admitted field returns to a raw primitive or gains an unchecked construction path
- **WHEN** compile-fail fixtures, source policy, or reviewed nominal-domain checks run
- **THEN** the regression MUST fail with a deterministic finding
- **AND** exceptions MUST identify a bounded compatibility boundary rather than disable the domain check broadly

### Requirement: Nominal type claims remain local

r[build_correctness.nominal_boundaries.claim_boundary] Mantle documentation and evidence MUST limit nominal-type claims to local scalar admission, category separation, relationship checks, and preserved wire identity.

#### Scenario: Admitted values pass all nominal checks

- **GIVEN** a boundary passes constructor, admission, role, compatibility, and policy checks
- **WHEN** Mantle reports that result
- **THEN** it MAY claim the supplied values satisfied the named local type contract
- **AND** it MUST NOT claim artifact correctness, source trust, store presence, safe filesystem I/O, sandbox enforcement, remote peer trust, compiler correctness, or release eligibility

### Requirement: Build layer maintains strict Tiger Style conformance

r[build_correctness.tiger_conformance] Mantle MUST keep `crunch-build` and `crunch-rustc-wrapper` within the complete pinned Tiger Style policy without lint allowances, warning budgets, finding baselines, target-scope reductions, or weaker full-check enforcement, while preserving derivation identity, canonical build inputs, content-addressed planning, typed profile admission, scheduler effect order, publication authority, and public compatibility.

#### Scenario: strict build check accepts both packages

GIVEN build-layer source uses checked arithmetic, bounded non-recursive processing, meaningful invariants, typed error propagation, decomposed conditions, and named interfaces
WHEN the focused package check and repository Tiger Style check run
THEN both MUST report zero `crunch-build` and `crunch-rustc-wrapper` findings without an allowance or suppressed target
AND positive and negative package tests, strict Clippy, and formatting MUST pass.

#### Scenario: structural repair would change build meaning

GIVEN a proposed lint repair changes canonical structured-attribute bytes, content-addressed output selection, profile admission, registry or scheduler order, no-replace publication, or public behavior without compatibility
WHEN the repair is reviewed or tested
THEN Mantle MUST reject the repair even if the Tiger Style command exits successfully
AND the finding MUST remain actionable until a semantics-preserving repair passes.

#### Scenario: full check advances beyond the build gate

GIVEN focused build validation and the repository Tiger Style check pass
WHEN local-builder and ordinary `nix flake check -L` run
THEN neither run MUST fail on a `crunch-build` or `crunch-rustc-wrapper` Tiger Style finding
AND any later independent failure MUST remain an exact blocker without disabling or downgrading its gate.
