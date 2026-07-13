# Hardware Simulation Builds Specification

## Purpose

Prove that Mantle’s frontend-neutral build primitives can realize a bounded, hermetic, selectively reusable SystemVerilog simulation and smoke-test graph without making HDL semantics part of Mantle core.

## Requirements

### Requirement: Hardware semantics remain outside Mantle core [r[hardware_simulation_builds.frontend_boundary]]

Mantle MUST represent the reference hardware flow through example- or frontend-owned typed data lowered to generic source, action, dynamic-plan, store, result, and evidence primitives. Mantle core MUST NOT interpret HDL modules, testbenches, verification IP, expected DUT behavior, simulator options, regression semantics, or coverage meaning.

#### Scenario: Hardware profile lowers to generic build inputs

- GIVEN a valid hardware reference profile names source packages, a top, tool cohort, generation options, and smoke cases
- WHEN the profile is lowered for Mantle
- THEN it MUST produce ordinary fixed sources, derivations, dynamic-plan units, action specs, and expected output artifacts
- AND no HDL-specific branch MAY be required in scheduler, store, sandbox, action-result, or output-admission core logic.

#### Scenario: Raw HDL policy reaches core

- GIVEN a caller asks Mantle core to infer module hierarchy, expected signal behavior, UVM semantics, or commercial-tool policy from source text
- WHEN the build-tool boundary validates the request
- THEN it MUST reject the unsupported semantic request or require an external frontend to lower it
- AND it MUST NOT silently add inferred files, options, authority, or expected behavior to the action.

### Requirement: Hardware reference profiles are typed and bounded [r[hardware_simulation_builds.typed_profile]]

The reference flow MUST use typed Nickel to validate source-package refs, tops, source sets, exact tool cohort, generation options, compile/link limits, smoke-case descriptors, expected outputs, support tier, and non-claims. Equivalent normalized profiles MUST produce the same profile identity, and unknown fields, duplicate names, escaping paths, empty required sets, or exceeded bounds MUST fail before execution.

#### Scenario: Equivalent profiles normalize identically

- GIVEN two profiles contain equivalent source, tool, option, bound, and smoke-case facts in different record order
- WHEN profile validation and canonicalization run
- THEN both MUST produce the same normalized plan and BLAKE3 profile identity
- AND map order, host paths, clocks, and ambient environment MUST NOT affect that identity.

#### Scenario: Profile exceeds a declared bound

- GIVEN a profile contains too many sources, generated units, options, smoke cases, outputs, or oversized text
- WHEN profile validation runs
- THEN it MUST fail with a stable bound diagnostic
- AND no source acquisition, tool execution, or store mutation may begin.

### Requirement: External hardware sources are exact and demand-driven [r[hardware_simulation_builds.demand_driven_sources]]

Mantle MUST acquire only the pinned fixed-output IP, VIP, model, and testbench source closures reachable from the selected hardware target. Source URL or repository locator, revision, hash mode, expected digest, and resulting object identity MUST be explicit; unrelated source packages MUST NOT be fetched or realized merely because they are present in the profile catalog.

#### Scenario: Selected testbench acquires only its closure

- GIVEN two testbenches depend on disjoint pinned IP/VIP source packages
- WHEN an operator builds only the first testbench
- THEN Mantle MUST request and realize only the first testbench’s reachable source closure
- AND evidence MUST show that the unrelated source fetch/build sentinel was not invoked.

#### Scenario: Pinned source drifts

- GIVEN source bytes, revision, recursive hash, or declared dependency edges differ from the admitted profile
- WHEN source acquisition or closure validation runs
- THEN Mantle MUST reject the source or produce a newly identified action graph
- AND it MUST NOT reuse an action result bound to the prior source identity.

### Requirement: The hardware tool cohort is exact and declared [r[hardware_simulation_builds.pinned_tool_cohort]]

Every real reference build MUST consume an exact declared Verilator, C++ compiler, linker, runtime-support, and shell cohort from store inputs and MUST bind a Mantle-owned BLAKE3 cohort identity into generation, compilation, linking, smoke, report, and result identities. Ambient PATH tools, user configuration, undeclared include roots, and implicit compiler defaults MUST NOT satisfy the strict profile.

#### Scenario: Declared cohort builds the simulator

- GIVEN every cohort member and required runtime input is present under its declared store ref
- WHEN generation, compilation, linking, and smoke actions run
- THEN each action MUST receive only the declared cohort closure
- AND reports MUST identify the cohort and any external seed boundary used to materialize it.

#### Scenario: Ambient tool would satisfy a missing input

- GIVEN a required cohort member is absent but a same-named executable exists on host PATH or in user configuration
- WHEN strict preflight or sandbox execution runs
- THEN Mantle MUST fail rather than use the ambient executable
- AND it MUST NOT publish the failed action as reusable hardware-build evidence.

### Requirement: Simulation construction is a staged bounded action graph [r[hardware_simulation_builds.staged_action_graph]]

The reference flow MUST separate SystemVerilog generation, generated translation-unit compilation, simulator linking, and smoke execution into independently identified actions. The generation action MAY emit a declared `mantle-plan-v1`, but every generated unit, dependency edge, path, command, output, collection size, and byte total MUST pass ordinary dynamic-plan validation before scheduling.

#### Scenario: Generated compilation is partitioned

- GIVEN a valid selected top and exact tool/source inputs produce multiple generated C++ translation units
- WHEN Mantle admits the declared dynamic plan
- THEN it MUST schedule bounded independent compile units and one link root over their exact outputs
- AND shared dependencies MUST deduplicate through the existing lazy goal registry.

#### Scenario: Generated plan escapes its authority

- GIVEN a generated plan names an undeclared file, escaping path, duplicate unit, unsupported command, missing dependency, undeclared output, or exceeded count/byte bound
- WHEN dynamic-plan admission runs
- THEN Mantle MUST reject the plan before registering its units
- AND no partial compile graph may be reported as admitted.

### Requirement: Smoke tests produce bounded admitted result artifacts [r[hardware_simulation_builds.smoke_results]]

Each smoke case MUST be a separate action over an exact simulator and canonical case descriptor. A result artifact MUST bind schema, case identity, bounded expected and observed values, verdict, process outcome, simulator ref, action ref, source/tool profile refs, bounded log refs, and non-claims. A passing result requires verdict and process outcome agreement; a failed or malformed case MUST NOT become a successful result.

#### Scenario: Smoke case passes

- GIVEN the simulator and case descriptor match the admitted profile and observed output matches the declared fixture expectation
- WHEN the smoke action exits successfully and its result validates
- THEN Mantle MAY admit the result artifact and ordinary action output
- AND the report MUST identify the case, action, simulator, profile, and bounded log refs.

#### Scenario: Reference model or result is wrong

- GIVEN the C++ model disagrees with the DUT fixture, the process exits unsuccessfully, or the result artifact claims a contradictory verdict
- WHEN result validation runs
- THEN Mantle MUST report the action as failed or the artifact as rejected
- AND it MUST NOT publish a passing smoke result or successful shared action result.

### Requirement: Hardware actions reuse only admitted shared results [r[hardware_simulation_builds.shared_reuse]]

Hardware generation, compile, link, and smoke actions MUST use Mantle’s generic shared action-result discovery and admission. A clean client MAY skip execution only after exact action, object completeness, PathInfo, receipt, signature, producer, sandbox/network policy, reference-scan, and claim-strength checks pass. HDL filenames, test names, store paths, or index presence alone MUST NOT authorize reuse.

#### Scenario: Clean client admits a matching smoke result

- GIVEN a fresh client has no local action mapping and discovers one complete trusted result for the exact smoke action
- WHEN shared-result admission succeeds
- THEN Mantle MAY fetch and admit the referenced objects without invoking the simulator
- AND the report MUST identify the selected result ref, trust basis, and zero executor calls.

#### Scenario: Same action has conflicting outputs

- GIVEN multiple otherwise admissible results for one hardware action name different output object sets
- WHEN strong reuse admission compares them
- THEN Mantle MUST fail with bounded conflicting-result evidence
- AND it MUST NOT select a result by arrival, source order, filename, or last writer.

### Requirement: Work-reduction evidence is explicit and bounded [r[hardware_simulation_builds.work_reduction_evidence]]

The reference rail MUST compare fresh, selected-source-change, unrelated-source-change, and full-shared-hit runs using a versioned bounded evidence bundle. It MUST record requested, executed, reused, and invalidated action counts by stage; transferred and reused bytes; relevant action/source/tool refs; environment class; and explicit non-claims. Elapsed measurements MAY be diagnostic but MUST NOT be a hard correctness threshold or be presented as reproducing external benchmark results.

#### Scenario: Selected source change invalidates dependents

- GIVEN one admitted RTL or model source changes while unrelated fixture and tool inputs remain fixed
- WHEN the comparison rail rebuilds the selected target
- THEN it MUST identify the generation, compile, link, and smoke actions whose exact input closure changed
- AND actions outside that dependency closure MUST remain reusable when their admission facts still pass.

#### Scenario: Full shared hit is reported honestly

- GIVEN every requested action has one complete admitted shared result on a clean client
- WHEN the reference rail runs
- THEN it MUST report no executor calls and bounded transfer/reuse facts
- AND it MUST NOT claim a specific speedup, production capacity, simulator-license saving, or remote-farm result from that fixture alone.

### Requirement: Hardware reference validation covers success and failure [r[hardware_simulation_builds.final_validation]]

The hardware reference change MUST include positive and negative pure-profile, source, plan, sandbox, tool-cohort, result, shared-reuse, invalidation, catalog, and documentation evidence. Heavy real-tool execution MUST have an explicit capability gate and command; missing capability MUST be reported rather than converted into a false passing integration claim.

#### Scenario: Reference change is ready for lifecycle review

- GIVEN hardware profile, source, planning, execution, result, reuse, or evidence behavior changes
- WHEN validation evidence is assembled
- THEN it MUST include focused positive and negative checks, exact commands and outputs, Cairn validation, proposal/design/tasks gates, and relevant first-party quality rails
- AND skipped real-tool execution MUST remain an explicit blocker for any claim requiring that execution.
