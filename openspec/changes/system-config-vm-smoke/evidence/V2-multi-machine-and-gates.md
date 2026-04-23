# Evidence placeholder: V2 multi-machine VM smoke and gates

Task-ID: V2
Covers: systemconfig.vm.testing.runtime.multi.machine.positive.negative, systemconfig.vm.testing.validation.heavy.rail
Status: pending

## Planned commands

- `nix build .#checks.x86_64-linux.vm-system-config-multi-machine`
- `openspec validate system-config-vm-smoke`
- `openspec_gate stage=design change=system-config-vm-smoke`
- `openspec_gate stage=tasks change=system-config-vm-smoke`

## Notes

Pending implementation.
