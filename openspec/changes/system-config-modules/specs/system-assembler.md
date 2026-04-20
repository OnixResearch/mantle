# System Assembler

## Overview

Converts a merged per-machine configuration tree into buildable
derivations using crunch's build pipeline. This is the imperative-shell
boundary — the only layer that touches the build engine and store.

## Requirements

### ASM-1: Assembler trait

The assembler MUST be defined as a Rust trait with at minimum:

```
fn name(&self) -> &str;
fn assemble(&self, machine_name: &str, machine: &MachineRecord,
            config: &MergedConfig)
    -> Result<Vec<CrunchDerivation>, AssemblerError>;
```

The `machine` parameter provides the full machine record from the
inventory, including `system`, `class`, and any user-defined metadata.
The trait MUST be object-safe for runtime dispatch.

### ASM-2: NixOS backend (phase 1 — structural passthrough)

A NixOS assembler backend MUST be provided as the first implementation.
Phase 1 scope: it MUST convert `output.nixos` fragments into a single
derivation per machine whose builder receives the merged NixOS config
tree as a JSON environment variable. The derivation's build script
writes the JSON to `$out/system-config.json`.

The NixOS backend MUST NOT shell out to `nix-build`, `nixos-rebuild`,
or any Nix CLI tool.

Full NixOS system closure assembly (generating `/etc`, systemd units,
activation scripts, initrd, kernel command line) is a follow-on change.
This change only proves the pipeline end-to-end: modules → evaluation
→ fragments → derivation → build.

### ASM-3: Backend registration

Assembler backends MUST be selectable by name (string). The pipeline
MUST support registering multiple backends. The active backend is chosen
per machine based on inventory metadata (e.g., `machine.class`).

### ASM-4: Derivation emission

The assembler MUST emit derivation records compatible with crunch's
existing `crunch-glue` conversion and `crunch-pipeline::build()`. It
MUST NOT bypass the standard build pipeline.

### ASM-5: Backend isolation

Each backend MUST be independent. Adding or removing a backend MUST NOT
require changes to the pipeline, evaluator, or collector layers. Backends
are separate Rust modules behind the trait.

### ASM-6: Dry-run support

The assembler trait MUST support a dry-run mode that returns derivation
records without submitting them to the build pipeline. This enables
`crunch system eval` to show what would be built.
