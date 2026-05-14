# system-assembler Specification

## Purpose

This spec defines assembler backends that turn merged machine configs into
`CrunchDerivation` values for the system-config pipeline.
## Requirements
### Requirement: ASM-1 Assembler trait

The system layer MUST define an object-safe assembler trait that converts a
merged machine configuration into one or more `CrunchDerivation` values.
ID: systemconfig.system.assembler.asm1

The trait MUST expose a backend name plus an `assemble(machine_name, machine,
config)` method that receives the full machine record and the merged config.
If a backend returns multiple derivations for one machine, later machine-level
reporting MUST preserve all of their build reports inside that machine's single
`MachineOutcome::Build { reports }` entry.

#### Scenario: Assembler backend receives the full machine record
ID: systemconfig.system.assembler.asm1.scenario

- GIVEN a merged config for `server1`
- WHEN an assembler backend runs
- THEN it receives `server1`'s full machine record
- AND it may inspect backend-specific metadata in that record

### Requirement: ASM-2 Phase-1 NixOS backend

The change MUST provide a first NixOS-oriented backend that proves the pipeline
end to end without invoking any Nix CLI tool.
ID: systemconfig.system.assembler.asm2

In phase 1, the backend MUST convert `output.nixos` fragments into a single
derivation per machine whose builder writes the merged NixOS config tree to
`$out/system-config.json`. The backend MUST NOT shell out to `nix-build`,
`nixos-rebuild`, or any other Nix command.

The end-to-end `mantle system eval` and `mantle system build` pipeline MUST NOT
require a Nix runtime dependency to be installed on the host. All required
behavior for this phase MUST come from mantle's own evaluator, assembler, and
build pipeline.

#### Scenario: NixOS backend emits one structural passthrough derivation
ID: systemconfig.system.assembler.asm2.scenario

- GIVEN a merged config containing `output.nixos`
- WHEN the phase-1 NixOS backend assembles the machine on a host without Nix
  installed
- THEN it returns one derivation whose builder writes `$out/system-config.json`
- AND no Nix CLI subprocess is invoked

### Requirement: ASM-3 Backend registration and selection

Assembler backends MUST be registered by string name, and backend selection
MUST default to each machine's inventory `class` value when no CLI override is
present.
ID: systemconfig.system.assembler.asm3

When `--assembler <name>` is supplied, the CLI override defined by CLI-5 MUST
take precedence for every selected machine. Backend lookup failures MUST be
reported as assembler diagnostics for the affected selected machines.

#### Scenario: Missing backend produces assembler diagnostics
ID: systemconfig.system.assembler.asm3.scenario

- GIVEN a selected machine whose effective backend name is not registered
- WHEN assembly begins
- THEN the machine fails with an assembler diagnostic naming the backend
- AND other selected machines may continue if their backends resolve

### Requirement: ASM-4 Derivation emission uses the standard pipeline

Assemblers MUST emit derivation records that are compatible with
`crunch-glue::convert()` and `crunch-pipeline::build()`.
ID: systemconfig.system.assembler.asm4

Assembler implementations MUST NOT bypass the standard build pipeline or invent
an alternative derivation execution path.

#### Scenario: Assembler output feeds crunch-pipeline unchanged
ID: systemconfig.system.assembler.asm4.scenario

- GIVEN a backend-produced `CrunchDerivation`
- WHEN `mantle system build` submits it for execution
- THEN the derivation flows through the same conversion and build pipeline used
  by existing mantle builds

### Requirement: ASM-5 Backend isolation

Assembler backends MUST remain isolated from evaluator, collector, and loader
logic.
ID: systemconfig.system.assembler.asm5

Adding or removing a backend MUST NOT require changes to the pipeline stages
outside the assembler registry and backend module set.

#### Scenario: New backend does not rewrite evaluator logic
ID: systemconfig.system.assembler.asm5.scenario

- GIVEN a new assembler backend is added to the registry
- WHEN the system-config pipeline is rebuilt
- THEN loader, evaluator, and collector behavior remains unchanged
- AND only assembler registration changes are required

### Requirement: ASM-6 Dry-run support

The assembler layer MUST support dry-run operation so that `mantle system eval`
can return derivation records without invoking the build pipeline.
ID: systemconfig.system.assembler.asm6

Dry-run mode MUST preserve the same backend selection rules as real builds.

#### Scenario: Eval uses assembler dry-run mode
ID: systemconfig.system.assembler.asm6.scenario

- GIVEN `mantle system eval <inventory.ncl>`
- WHEN assembly completes in dry-run mode
- THEN stdout contains derivation records rather than built outputs
- AND no derivation is submitted to `crunch-pipeline::build()`

