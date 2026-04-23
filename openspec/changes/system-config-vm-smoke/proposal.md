## Why

`crunch system` now validates inventories, merges module fragments, and can dry-
run or build per-machine derivations, but current coverage still stops at JSON
and derivation envelopes. We do not yet prove that the checked-in
`output.nixos` fragments produced by crunch actually evaluate, boot, and behave
correctly as running systems.

That gap matters for the new module pipeline. A module can pass Rust-side CLI
and merge tests while still failing later at NixOS evaluation, service
activation, or cross-machine reachability. We need a runtime smoke layer for
our checked-in modules and system configs before the system-config surface can
be treated as trustworthy.

## What Changes

- **Add a checked-in VM smoke harness.** Run `crunch system eval <inventory>
  --stop-after fragments`, extract each successful machine's
  `merged_config.data.output.nixos`, and feed that data into NixOS VM tests.
- **Boot live system-config fixtures.** Start with the checked-in
  `examples/system-config/` inventory and modules so the VM layer exercises the
  same source fixtures already covered by `tests/system_cli.rs`.
- **Cover positive and negative runtime behavior.** Add both a single-machine
  service smoke path and a multi-machine reachability check with an expected
  denied connection.
- **Register the suite as a heavyweight rail.** Expose the VM smoke checks as
  named flake checks and document them separately from ordinary Rust-only
  validation.

## Non-Goals

- Full NixOS closure assembly or deployment from crunch.
- A new end-user `crunch system test` CLI command.
- Non-NixOS backends or non-Linux VM runners.
- Replacing the existing Rust integration tests for CLI and evaluator logic.

## Capabilities

### New Capabilities
- `system-config-vm-bridge`: convert live `crunch system eval --stop-after
  fragments` output into NixOS VM node imports.
- `system-config-vm-smoke`: boot checked-in module/inventory fixtures and
  assert runtime behavior.
- `system-config-vm-heavy-gate`: register the VM suite as a named heavyweight
  validation rail.

## Impact

- **Files**: new `tests/system-vm/` Nix helpers/tests, `flake.nix`, and docs
  describing the heavyweight VM commands; `examples/system-config/` may gain
  small fixture adjustments only if needed for runtime-visible assertions.
- **APIs**: no new end-user crunch CLI surface; test-only bridge helpers only.
- **Dependencies**: reuses the built `crunch` binary plus NixOS test infra
  (`pkgs.testers.runNixOSTest`) for verification only.
- **Testing**: adds KVM-backed NixOS VM smoke coverage on top of the existing
  Rust integration layer.

## Constraints

- The VM harness MUST consume live crunch output, not hand-copied NixOS module
  fragments.
- Test-only Nix and KVM usage MUST NOT widen the runtime claim that
  `crunch system` itself remains Nix-free.
- The initial suite MUST include both positive and negative runtime assertions.
- Ordinary first-party edit-time validation MUST NOT require the VM rail.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| VM bridge, fixture reuse, runtime smoke, and heavyweight gate | `specs/system-vm-testing/spec.md` |

## How to validate

1. `openspec validate system-config-vm-smoke` succeeds.
2. `openspec_gate stage=proposal change=system-config-vm-smoke` passes.
3. `cargo test -p crunch --test system_cli system_eval_stop_after_fragments_returns_merged_configs -- --nocapture`
   still proves the JSON bridge source shape used by the VM harness.
4. `nix build .#checks.x86_64-linux.vm-system-config-server1-smoke` boots a
   single-machine VM from live crunch fragments and verifies service behavior.
5. `nix build .#checks.x86_64-linux.vm-system-config-multi-machine` boots the
   checked-in two-machine example inventory and verifies both an allowed and a
   denied cross-machine HTTP path.
6. `openspec_gate stage=design change=system-config-vm-smoke` and
   `openspec_gate stage=tasks change=system-config-vm-smoke` pass once the
   design and task packet are complete.
