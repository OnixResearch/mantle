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
