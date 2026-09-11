# Specification: Derivation finish gates

## ADDED Requirements

### Requirement: Gates are one shared contract

r[mantle.derivation_finish_gates.shared_contract] Mantle MUST define a
versioned finish-gate contract in typed Nickel policy and consume it through a
deterministic export in the shell.

The contract MUST name each gate, its inputs, its default state, and its
explicit opt-out. Gates MUST run after install in one shared finish step owned
by the builder layer, not per-recipe shell. A gate result MUST be recorded in
the build report.

#### Scenario: A new derivation declares nothing

- GIVEN a derivation built through the builder layer with no gate fields
- WHEN the finish step runs
- THEN the default gate set runs and the build report names each gate result.

#### Scenario: The policy export is not fresh

- GIVEN a changed finish-gate policy whose deterministic export was not
  regenerated
- WHEN the shell loads the contract
- THEN Mantle MUST fail with the policy freshness blocker instead of running a
  stale gate set.

### Requirement: Version check gate

r[mantle.derivation_finish_gates.version_check] The finish step MUST run a
declared command in an empty environment and require its output to contain the
pinned version.

The command MUST default to `bin/<name> --version` for derivations that install
a binary. A missing binary, a nonzero exit, or output without the pinned
version MUST fail the build. Derivations without a runnable binary MUST declare
an explicit opt-out.

#### Scenario: Correct version

- GIVEN an output whose declared command prints the pinned version
- WHEN the version gate runs
- THEN the gate passes and the report records the command.

#### Scenario: Wrong or missing version

- GIVEN an output whose command exits nonzero, prints nothing, or prints a
  different version
- WHEN the version gate runs
- THEN the build MUST fail with the command, exit status, and expected string.

### Requirement: Reference leak gate

r[mantle.derivation_finish_gates.reference_leak_gate] The finish step MUST scan
outputs for absolute references using the configured logical store prefix and
apply the declared policy.

In a cross build, any reference to a build-platform path MUST fail the build.
In a native build, absolute store-prefix references MUST be reported per policy
with the offending file and a bounded context excerpt. The gate MUST NOT
rewrite references; reporting and denial only.

#### Scenario: Cross build leaks the build platform

- GIVEN a cross output containing a build-platform store path
- WHEN the leak gate runs
- THEN the build MUST fail naming the file and the leaked reference.

#### Scenario: Native build contains its own absolute prefix

- GIVEN a native output containing an absolute reference under the logical
  prefix
- WHEN the leak gate runs under report policy
- THEN the gate MUST report the file and excerpt without failing the build.

### Requirement: Relocation rerun gate

r[mantle.derivation_finish_gates.relocated_rerun] When enabled, the finish step
MUST copy the output to another path and rerun the version check from there.

The rerun MUST execute with an empty environment and a working directory
outside the build tree. Failure MUST fail the build. The gate MUST be opt-in
because it costs a copy of the output.

#### Scenario: Relocated output still runs

- GIVEN a passing version gate and an opt-in relocation gate
- WHEN the copied output runs the same command
- THEN the rerun MUST pass.

#### Scenario: Relocated output fails

- GIVEN an output that only works from its build path
- WHEN the copied output runs the same command
- THEN the build MUST fail with the copied path and the command output.

### Requirement: Dlopen audit gate

r[mantle.derivation_finish_gates.dlopen_audit] When enabled, the version-check
run MUST execute under a loader audit that records unresolved dlopen lookups.

A dlopen by soname that finds no provider MUST fail the build unless the recipe
declares that soname optional. Declared optional sonames MUST appear in the
gate result. The gate MUST be limited to Linux dynamic outputs.

#### Scenario: Undeclared missing dlopen provider

- GIVEN a dynamic output whose version-check run dlopens a library absent from
  its closure
- WHEN the audit gate runs
- THEN the build MUST fail naming the unresolved soname.

#### Scenario: Declared optional soname

- GIVEN the same output with the soname declared optional
- WHEN the audit gate runs
- THEN the gate passes and the report names the skipped soname.
