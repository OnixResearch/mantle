# Evidence placeholder: V1 bridge and single-machine VM smoke

Task-ID: V1
Covers: systemconfig.vm.testing.bridge.eval.fragments, systemconfig.vm.testing.fixtures.live.crunch.output, systemconfig.vm.testing.runtime.single.machine.smoke
Status: pending

## Planned commands

- `cargo test -p crunch --test system_cli system_eval_stop_after_fragments_returns_merged_configs -- --nocapture`
- bridge preflight command or check (to be filled in during implementation)
- `nix build .#checks.x86_64-linux.vm-system-config-server1-smoke`

## Notes

Pending implementation.
