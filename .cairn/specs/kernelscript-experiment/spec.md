# Kernelscript Experiment Specification

## Purpose

Defines the `kernelscript-experiment` capability.

## Requirements

### Requirement: KernelScript use is an explicit bounded experiment

r[kernelscript_experiment.profile] Mantle MUST require a non-default typed `mantle-kernelscript-experiment-v1` profile binding KernelScript source, exact compiler revision, dependency/toolchain cohort, target architecture and Onix kernel-build identity, BTF/header/config refs, selected output classes, expected generated files, named bounds, and beta/non-production non-claims.

#### Scenario: Complete experiment profile is admitted

- GIVEN a profile names every source, compiler, target, output, bound, and non-claim field with valid typed identities
- WHEN profile validation runs
- THEN Mantle MAY plan the experiment under that exact profile identity
- AND no default package, release, or OnixOS target may be enabled by the profile.

#### Scenario: Profile relies on ambient kernel inputs

- GIVEN a profile omits a required target identity or asks to use the running host's implicit BTF, headers, config, architecture, or kernel release
- WHEN admission runs
- THEN it MUST fail before code generation or compilation.

### Requirement: KernelScript compiler materialization is pinned and offline

r[kernelscript_experiment.compiler] Mantle MUST materialize the KernelScript compiler from a pinned fixed-output source and locked OCaml/dune/opam dependency closure, MUST run compiler build and code generation without network access, and MUST bind compiler binary and closure bytes with BLAKE3.

#### Scenario: Pinned compiler builds

- GIVEN all fixed-output sources and locked dependencies match their declared identities
- WHEN the compiler derivation runs in the Mantle sandbox
- THEN it MUST produce the exact compiler cohort identity used by experiment receipts.

#### Scenario: Source, dependency, or network behavior drifts

- GIVEN a source hash differs, a dependency is undeclared, the compiler attempts network access, or the resulting compiler identity differs
- WHEN materialization runs
- THEN the experiment MUST fail without using ambient opam state or a host `kernelscript` binary.

### Requirement: Code generation is a separate evidenced stage

r[kernelscript_experiment.codegen] Mantle MUST run KernelScript code generation separately from native compilation, MUST retain every bounded generated file, and MUST validate the generated project against an exact expected-file and file-class manifest before downstream planning.

#### Scenario: Generated project matches profile

- GIVEN the pinned compiler transforms the admitted `.ks` source into only expected userspace, eBPF, optional module, test, Makefile, and documentation file classes
- WHEN generated-project classification runs
- THEN it MUST produce a canonical generated-file manifest with per-file BLAKE3 identities
- AND downstream compilation MAY consume only admitted source classes.

#### Scenario: Generated layout drifts

- GIVEN an expected file is missing, an unknown executable/source file appears, a path escapes the output root, or a generated file exceeds a named bound
- WHEN classification runs
- THEN compilation MUST be blocked and the drift MUST remain explicit evidence.

### Requirement: Mantle owns explicit compilation plans

r[kernelscript_experiment.artifacts] Mantle MUST derive allowlisted userspace, eBPF, optional module, and test compilation steps from the admitted profile and generated-file manifest, MUST classify output sets independently, and MUST NOT execute the generated Makefile or unreviewed generated shell commands.

#### Scenario: Selected output classes build

- GIVEN every generated source and explicit toolchain input is admitted
- WHEN the Mantle-owned compilation plan runs
- THEN each selected output class MUST receive independent exact member identities and build/static-inspection outcomes
- AND success in one class MUST NOT mask a failure in another.

#### Scenario: Generated Makefile is selected for execution

- GIVEN any plan or shell attempts to invoke the generated Makefile or a command not in the explicit allowlist
- WHEN command admission runs
- THEN it MUST fail before execution
- AND the Makefile MAY remain only as retained review evidence.

### Requirement: Privileged artifacts bind exact kernel inputs

r[kernelscript_experiment.kernel_inputs] Every eBPF or module candidate MUST bind the exact target architecture, Onix kernel-build identity, BTF, kernel headers/config, compiler flags, and relevant toolchain identities, and missing or mismatched required inputs MUST block that artifact class.

#### Scenario: Kernel inputs agree

- GIVEN eBPF/module plans reference exact admitted BTF, headers/config, release, architecture, and kernel-build identities
- WHEN target admission runs
- THEN the selected artifact class MAY compile and receive static metadata inspection.

#### Scenario: BTF or headers do not match

- GIVEN required BTF is absent, module headers/config describe another kernel, architecture/release differs, or input identity changed after planning
- WHEN target admission or output verification runs
- THEN the affected artifact class MUST fail without substituting running-host inputs.

### Requirement: KernelScript outputs remain candidate handoffs

r[kernelscript_experiment.handoff] Mantle MAY emit frontend-neutral experimental ModulePack/BPF Pack candidate projections only after exact member and manifest validation, and every projection MUST remain `experimental-unverified` until separate Onix semantic and ChaosControl runtime gates pass.

#### Scenario: Candidate pack is emitted

- GIVEN selected module/BPF artifacts and target bindings pass build and static shape checks
- WHEN handoff projection runs
- THEN it MUST preserve exact target, member, manifest, and evidence identities
- AND it MUST NOT report verifier acceptance, successful load/attach, deployability, or production support.

#### Scenario: Runtime evidence is absent

- GIVEN a candidate object compiles but has no matching ChaosControl load/attach evidence
- WHEN readiness is summarized
- THEN the artifact MUST remain experimental-unverified.

### Requirement: KernelScript experiment evidence is bounded

r[kernelscript_experiment.evidence] Experiment receipts MUST bind source, compiler/dependency closure, target kernel inputs, codegen command identity, generated-file manifest, explicit compilation plans, output identities, static inspections, blockers, and non-claims with BLAKE3 while excluding source bodies, raw logs, full host paths, credentials, and unbounded diagnostics.

#### Scenario: Receipt is reconstructed

- GIVEN an experiment completes or blocks at a typed stage
- WHEN receipt projection runs
- THEN it MUST identify the exact replay cohort and per-output status
- AND it MUST state that beta build evidence is not language soundness, verifier acceptance, kernel safety, runtime correctness, or production readiness.

### Requirement: KernelScript experiment has positive and negative evidence

r[kernelscript_experiment.verification] The experiment MUST include deterministic positive and negative profile, compiler, codegen, explicit planner, build, static-inspection, handoff, receipt, dependency-audit, and overclaim fixtures.

#### Scenario: Experiment change is ready to archive

- GIVEN maintainers intend to close the bounded KernelScript experiment change
- WHEN closeout validation runs
- THEN focused positive and negative checks, first-party quality checks, docs, Cairn validation, and proposal/design/tasks gates MUST pass
- AND no generated artifact may be promoted beyond the evidence actually produced.
