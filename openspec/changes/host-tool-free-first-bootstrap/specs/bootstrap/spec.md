## ADDED Requirements

### Requirement: Stage0 protected phase forbids undeclared host executables

Crunch MUST define a first-bootstrap protected phase in which the stage0 crunch process does not execute undeclared host commands before crunch-built tools are available.
ID: bootstrap.hosttoolfree.exec.boundary

The protected phase MUST begin at stage0 `crunch self-build` or no-host-tools proof process entry before any source staging, fetch, validation, sandbox startup, or bootstrap-tool build work. It MUST end only after the exact crunch-built bootstrap tool outputs for `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl` have been built, exported, verified, and selected as the sandbox entry and sandbox shell for later stages; until that transition event occurs, every process launch is protected-phase execution.

Allowed protected-phase execution MUST be limited to the stage0 crunch binary itself, direct kernel interfaces, and executable seed artifacts declared in the stage0 inventory with digests. Direct kernel interfaces mean syscalls made by the stage0 process or its linked libraries without resolving or executing another filesystem path: file metadata/content I/O under declared workspace, store, state, proof, request, and scratch roots; deterministic hashing and serialization; network fetches only for declared bootstrap source artifacts; direct namespace, mount, chroot/pivot-root, uid/gid-map, pipe, socketpair, epoll/poll, wait, signal, mmap, futex, and clone/fork primitives needed to create a sandbox; and `execve` only when the target is the stage0 crunch binary itself or a declared seed executable with an allowed protected-phase role whose digest has already matched the inventory. Allowed protected-phase executable seed roles MUST be limited to `sandbox-entry`, `sandbox-shell`, `bootstrap-toolchain-tool`, and `bootstrap-build-tool`. Host commands such as `git`, `tar`, `cp`, `cargo`, ad hoc helper commands, host `sh`, and host `bwrap` MUST NOT be valid protected-phase seed identities and MUST NOT be invoked unless the executable path is a separately declared seed artifact with one of the allowed protected-phase roles and non-host provenance. Nix commands including `nix-build`, `nix-store`, `nix-shell`, `nix`, and `nix develop` MUST NOT be valid protected-phase seed executables and MUST NOT be invoked even if a matching digest is supplied.

The stage0 inventory schema MUST be the single source of truth for protected-phase executable seed artifacts and protected-phase non-executable bootstrap source artifacts. Each executable entry MUST include `schema_version`, stable `id`, `role`, `phase`, absolute `executable_path`, `digest.algorithm`, `digest.hex`, `provenance`, `allowed_reason`, `owner`, and `required` fields. Each non-executable source entry that may be fetched in the protected phase MUST include `schema_version`, stable `id`, `role`, `phase`, `urls`, `digest.algorithm`, `digest.hex`, `provenance`, `allowed_reason`, `owner`, `required`, and extraction rules. `digest.algorithm` MUST be `blake3` unless the entry includes an interoperability reason naming the external format that requires another algorithm. `phase` MUST distinguish protected-phase seed use from post-bootstrap crunch-built outputs. Missing required entries, digest mismatches, undeclared executable paths or source URLs, and entries without provenance or allowed reason MUST fail closed before execution or fetch.

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

The proof bundle MUST include a machine-readable execution audit, the stage0 inventory digest, the blocked host command set, explicit `self-build-proof: fallback-event=...` markers for protected-phase host-tool fallback decisions, and the final result. The mode MUST keep the existing self-hosting fixed-point checks for stage1, stage2, busybox, and bwrap.

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
