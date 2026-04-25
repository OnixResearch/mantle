## Why

`crunch-system` currently hard-codes NixOS concepts: the default assembler is named `"nixos"`, the assembler extracts `output.nixos` from merged configs, test fixtures reference `"class": "nixos"`, and the output is a `system-config.json` that assumes NixOS consumption. This couples crunch to NixOS even though the goal is an independent system — onixos — built entirely by crunch using the onix module system as an optional integration.

The onix module system (sibling `onix-modules` repo) already has 19 service modules that produce structured config fragments. Its architecture is clean: Nickel owns configuration, and a build backend (currently a Nix shim in `nix/assemble.nix`) turns fragments into a bootable system. crunch should be that build backend, but it should not hard-couple to onix — the module system is optional, and `crunch system` should work with any module source that produces the expected fragment shape.

## What Changes

- **Strip NixOS naming from crunch-system**: Rename the default assembler from `"nixos"` to something generic (e.g. `"system"` or make it configurable). Remove hard-coded `output.nixos` extraction — the assembler should work with whatever output namespace the modules produce.
- **Generic assembler trait**: The existing `Assembler` trait is already generic. Clean up `NixosPhase1Assembler` to not assume NixOS-shaped output. The assembler should accept a configurable output namespace or produce a generic config bundle.
- **onixos assembler backend**: A new assembler (potentially in a separate crate or behind a feature flag) that knows how to take merged onix module fragments and produce a bootable system — init scripts, service definitions, filesystem layout, kernel/initrd references. This is the piece that replaces `nix/assemble.nix` from onix-modules.
- **No hard coupling to onix**: `crunch system eval` and `crunch system build` work without onix installed. If onix modules are present, they're loaded as regular Nickel files through the existing module loader. The assembler is selected by inventory `class` field, not hard-coded.
- **Inventory class field**: Machines declare `class = "onixos"` (or any string) and the assembler registry dispatches to the matching backend. Unknown class is an error.

## Capabilities

### New Capabilities
- `onixos-assembler`: Assembler backend that produces bootable onixos system closures from merged module fragments
- `generic-assembler-dispatch`: Class-based assembler selection from inventory
- `system-apply`: Eventually, `crunch system apply` to activate a built system config on a running machine (stretch goal)

### Modified Capabilities
- `system-eval`: No longer assumes NixOS output shape
- `system-build`: Routes through class-based assembler dispatch

### Removed Capabilities
- `nixos-phase1-assembler`: The current NixOS-specific assembler is removed or relegated to a legacy/compat backend

## Impact

- **Files**: `crates/crunch-system/src/assembler.rs` (refactor), `crates/crunch-system/src/evaluator.rs` (strip NixOS refs), `crates/crunch-system/src/collector.rs` (generic output namespaces), test fixtures, `lib/system_module.ncl` (rename `nixos` output field), `lib/inventory.ncl`
- **APIs**: `Assembler` trait stays but `NixosPhase1Assembler` is replaced. `AssemblerRegistry` dispatch changes.
- **Dependencies**: No new Rust deps. onix-modules stays a separate repo — integration is through Nickel imports, not Rust crate deps.
- **Testing**: Existing system-config tests updated to use generic naming. New tests for class-based assembler dispatch. Integration test with sample onix-style modules.
