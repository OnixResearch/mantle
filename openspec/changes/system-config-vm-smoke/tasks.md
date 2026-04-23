# Tasks: system-config VM smoke

## Phase 1: Bridge crunch fragment output into VM nodes

- [ ] I1 Add a checked-in Nix helper under `tests/system-vm/` that runs the
      built `crunch` binary with `system eval <inventory> --stop-after
      fragments`, parses the JSON envelope, extracts
      `machines.<name>.merged_config.data.output.nixos`, and rejects selected
      machine entries that are missing, failed, or missing `merged_config` /
      `output.nixos` before VM boot starts. [covers=systemconfig.vm.testing.bridge.eval.fragments,systemconfig.vm.testing.fixtures.live.crunch.output]
- [ ] I2 Add a thin node-wrapper layer for VM smoke checks that imports the
      extracted fragment data and only adds test-observation helpers such as
      headless VM defaults and packages like `curl`, without restating the
      service behavior that crunch is supposed to provide. [covers=systemconfig.vm.testing.fixtures.live.crunch.output]

## Phase 2: Boot the checked-in example system config

- [ ] I3 Add `vm-system-config-server1-smoke` that boots `server1` from the
      checked-in `examples/system-config/` fixture, waits for the boot target,
      checks `openssh.service`, verifies the configured SSH port, and confirms
      nginx answers on `http://localhost:8080/`. [covers=systemconfig.vm.testing.runtime.single.machine.smoke]
- [ ] I4 Add `vm-system-config-multi-machine` that boots `server1` and
      `server2` from the checked-in example inventory, proves
      `server2 -> server1:8080` succeeds, and proves
      `server1 -> server2:8081` fails as the expected negative assertion
      derived from `server2`'s example firewall settings. [covers=systemconfig.vm.testing.runtime.multi.machine.positive.negative]

## Phase 3: Wire the heavyweight validation rail

- [ ] I5 Register `vm-system-config-server1-smoke` and
      `vm-system-config-multi-machine` as named `flake.nix` checks and keep
      them out of the ordinary Rust-only edit-time quality path.
      [covers=systemconfig.vm.testing.validation.heavy.rail]
- [ ] I6 Document the exact VM smoke commands and prerequisites in the checked-
      in docs so contributors can discover the KVM-backed rail without reading
      Nix code. [covers=systemconfig.vm.testing.validation.heavy.rail]

## Validation

- [ ] V1 Run `cargo test -p crunch --test system_cli
      system_eval_stop_after_fragments_returns_merged_configs -- --nocapture`
      plus the new VM-bridge preflight check, then run `nix build
      .#checks.x86_64-linux.vm-system-config-server1-smoke` and capture the
      output proving the live crunch fragment bridge and single-machine boot
      path work together. [covers=systemconfig.vm.testing.bridge.eval.fragments,systemconfig.vm.testing.fixtures.live.crunch.output,systemconfig.vm.testing.runtime.single.machine.smoke] [evidence=openspec/changes/system-config-vm-smoke/evidence/V1-bridge-and-single-machine.md]
- [ ] V2 Run `nix build
      .#checks.x86_64-linux.vm-system-config-multi-machine`, confirm the
      positive and negative cross-machine assertions, then rerun `openspec
      validate system-config-vm-smoke`, `openspec_gate stage=design
      change=system-config-vm-smoke`, and `openspec_gate stage=tasks
      change=system-config-vm-smoke`. [covers=systemconfig.vm.testing.runtime.multi.machine.positive.negative,systemconfig.vm.testing.validation.heavy.rail] [evidence=openspec/changes/system-config-vm-smoke/evidence/V2-multi-machine-and-gates.md]
