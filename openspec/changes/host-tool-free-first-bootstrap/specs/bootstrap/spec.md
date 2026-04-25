## ADDED Requirements

### Requirement: Stage0 protected phase forbids undeclared host executables

Crunch MUST define a first-bootstrap protected phase in which the stage0 crunch process does not execute undeclared host commands before crunch-built tools are available.
ID: bootstrap.hosttoolfree.exec.boundary

Allowed protected-phase execution MUST be limited to the stage0 crunch binary itself, direct kernel interfaces, and executable seed artifacts declared in the stage0 inventory with digests. Host commands such as `git`, `tar`, `cp`, `sh`, `cargo`, `bwrap`, `nix-build`, `nix-store`, `nix-shell`, and `nix develop` MUST NOT be invoked in the protected phase unless they are declared seed artifacts with digests.

#### Scenario: Fake host commands are not invoked

- GIVEN `PATH` contains fake `git`, `tar`, `cp`, `sh`, `cargo`, `bwrap`, and Nix
  commands that fail loudly when executed
- WHEN the protected first-bootstrap phase runs
- THEN none of those fake commands are invoked
- AND the phase either succeeds or fails only on declared missing seed artifacts

#### Scenario: Undeclared executable path is rejected

- GIVEN bootstrap code attempts to execute `/usr/bin/tar` during the protected
  phase
- WHEN the execution audit checks the path
- THEN bootstrap fails before executing it
- AND the diagnostic names `/usr/bin/tar` as undeclared

### Requirement: First bootstrap has a native sandbox entry point

Crunch MUST provide a protected first-bootstrap sandbox entry point that does not require executing host `bwrap` before a crunch-built or declared-seed sandbox tool exists.
ID: bootstrap.hosttoolfree.sandbox.entrypoint

The protected entry point MAY use Rust-owned namespace/syscall setup directly, or it MAY execute a declared seed sandbox binary whose path and digest are recorded in the stage0 inventory. It MUST NOT silently fall back to host `bwrap` or host shell execution.

#### Scenario: Declared seed sandbox is accepted

- GIVEN the stage0 inventory declares a sandbox executable path and digest
- AND the executable bytes match that digest
- WHEN first bootstrap starts the protected sandbox
- THEN the sandbox may be used
- AND the proof report records it as a declared seed artifact

#### Scenario: Host bwrap fallback is rejected

- GIVEN no declared sandbox seed exists
- AND host `bwrap` is present on `PATH`
- WHEN first bootstrap starts in host-tool-free mode
- THEN it fails before sandbox startup
- AND the diagnostic says host `bwrap` fallback is forbidden in this mode

### Requirement: No-host-tools proof mode audits protected execution

Crunch MUST provide a no-host-tools proof mode that records every protected-phase executable and fails if any executable is not the stage0 crunch binary or a declared seed artifact.
ID: bootstrap.hosttoolfree.proof.mode

The proof bundle MUST include a machine-readable execution audit, the stage0 inventory digest, the blocked host command set, and the final result. The mode MUST keep the existing self-hosting fixed-point checks for stage1, stage2, busybox, and bwrap.

#### Scenario: Audit records only allowed executables

- GIVEN a no-host-tools proof run completes
- WHEN the proof bundle is inspected
- THEN the execution audit lists only the stage0 crunch binary and declared seed
  artifact executables before crunch-built tools become available
- AND the bundle still records stage1==stage2 fixed-point status

#### Scenario: Hidden host tool fails the proof

- GIVEN a protected-phase code path tries to execute an undeclared host command
- WHEN no-host-tools proof mode runs
- THEN the proof fails
- AND the proof bundle identifies the hidden host tool
