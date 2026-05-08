# V1 prerequisite runtime evidence audit

Task-ID: V1
Covers: bootstrap.source.chain.runtime-validation

## Result

The source-chain umbrella cannot yet consume completed runtime-validation proof for the compiler transition chain. The three prerequisite runtime-validation changes are present and strict-OpenSpec-valid, but all remain active with no completed tasks and no committed evidence directories:

- `live-bootstrap-binutils-tcc-chain-runtime-validation`: blocker remains the binutils-tcc chain validation itself. Its task list is `done=0, todo=6`; V1-V6 are still open, including rerunning preflight, completing/splitting the `bootstrap/bzip2-tcc.ncl` prerequisite, epoch builds, host-leakage audit, post-musl linkage checks, and OpenSpec validation.
- `live-bootstrap-gcc-4-0-runtime-validation`: blocked on binutils-tcc runtime proof. Its task list is `done=0, todo=5`; V1 explicitly requires binutils-tcc evidence or its blocker before building/smoking `bootstrap/gcc-4.0.ncl`.
- `live-bootstrap-gcc-4-7-runtime-validation`: blocked on both binutils-tcc and gcc-4.0 runtime proof. Its task list is `done=0, todo=5`; V1 explicitly requires binutils-tcc and gcc-4.0 evidence or blockers before building/smoking `bootstrap/gcc-4.7.ncl`.

This satisfies the umbrella V1 audit requirement by recording the current blockers. It does **not** satisfy umbrella V2-V4 runtime proof; those remain pending until the child runtime-validation changes produce build transcripts, leakage scans, and compiler/linker smoke evidence.

## Commands run

```sh
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py drain-plan
openspec validate live-bootstrap-binutils-tcc-chain-runtime-validation --strict --json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-binutils-tcc-chain-runtime-validation --json
openspec validate live-bootstrap-gcc-4-0-runtime-validation --strict --json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-gcc-4-0-runtime-validation --json
openspec validate live-bootstrap-gcc-4-7-runtime-validation --strict --json
python /home/brittonr/.hermes/skills/agentkit-port/openspec/scripts/openspec_helper.py verify live-bootstrap-gcc-4-7-runtime-validation --json
```

## Transcript summary

- `openspec validate ... --strict --json` passed for all three child changes with no issues.
- The helper `verify` command returned warning-only incomplete-task reports:
  - binutils-tcc: `tasks incomplete: {'done': 0, 'todo': 6, 'in_progress': 0}`
  - gcc-4.0: `tasks incomplete: {'done': 0, 'todo': 5, 'in_progress': 0}`
  - gcc-4.7: `tasks incomplete: {'done': 0, 'todo': 5, 'in_progress': 0}`
- File inventory confirmed each child change currently contains only `.openspec.yaml`, `proposal.md`, `design.md`, `tasks.md`, and `specs/bootstrap/spec.md`; no `evidence/` directory is present yet.

## Next blocker to clear

Start with `live-bootstrap-binutils-tcc-chain-runtime-validation` V1/V2: rerun build preflight with `nix shell nixpkgs#bubblewrap` and writable local store/state directories, then complete or split the `bootstrap/bzip2-tcc.ncl` prerequisite build and record the Mes runtime boundary.
