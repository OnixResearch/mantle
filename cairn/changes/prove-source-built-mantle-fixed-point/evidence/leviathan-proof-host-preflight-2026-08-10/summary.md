# Leviathan proof-host preflight

Task-ID: V2
Covers: bootstrap_inventory.source_built_mantle_fixed_point

## Result

Leviathan (`leviathan.cymric-daggertooth.ts.net`) is the selected execution host for the first promoted V2 source-built Mantle fixed-point run.

The observed host has 256 CPUs, 263,485,988 KiB of memory, 1,795,028,868 KiB free on the root filesystem, and a 524,288 hard open-file limit. It runs Linux 6.18.38 on x86-64. The kernel configuration enables user namespaces, seccomp, and seccomp filters. The explicit bubblewrap 0.11.2 sandbox probe and static BusyBox probe passed.

The host cannot authenticate to GitHub because its configured SSH identity is absent. The operator must transfer the exact source tree and authenticated source profile through the existing Tailscale SSH route. The final V2 evidence must retain post-transfer parity before proof launch.

## Oracle checkpoint

| Field | Value |
|---|---|
| Question | Which host will execute the first promoted source-built Mantle fixed-point proof? |
| Inspected evidence | Current Leviathan system, resource, kernel-config, bubblewrap, static-shell, pueue, and GitHub SSH probes in `preflight.log`; local-only proof rules in the change design and ADR 0070. |
| Decision | Use Leviathan as the sole six-stage execution host. Keep SSH, rsync, and pueue outside the proof action graph as transfer and control mechanisms. |
| Owner | I3 through I5 implement and bind the proof. V2 runs it on Leviathan and retains host and transfer evidence. |
| Next action | Complete source-transfer parity, build the exact proof revision on Leviathan, generate a matching authenticated profile, run the focused seccomp supervisor rail, and launch V2 through Leviathan's pueue daemon. |

## Authority boundary

The Tailscale DNS name selects the operator route. It does not grant proof authority.

Mantle `--builder`, Nix remote-action dispatch, split-host stages, development-cache completion, and the route name cannot satisfy the proof. All six stages must execute locally on Leviathan under the authenticated plan and runtime reconciliation.

## Non-claims

This preflight proves only the listed host observations and two sandbox-tool probes. It does not prove seccomp user-notification behavior, source-transfer parity, profile freshness, provider construction, stage1 or stage2 success, fixed-point equality, or release eligibility.

## Validation

Repository-pinned Cairn revision `fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8` passed strict validation and the proposal, design, and tasks gates. `git diff --check` also passed.

## Transcripts

The exact host output is in `preflight.log`. The exact Cairn outputs are in `cairn-validate.log`, `cairn-proposal-gate.log`, `cairn-design-gate.log`, and `cairn-tasks-gate.log`.
