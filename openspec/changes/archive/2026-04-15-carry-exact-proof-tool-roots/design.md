# Design: Carry exact proof tool roots into later self-build stages

## Summary

Add hidden proof-only self-build arguments for the exact stage0-produced
`bwrap` and `busybox` binaries. The self-hosting proof passes those exact paths
from stage0 to stage2.

Stage2 then:

- activates the handed-off `bwrap` path before later builds start,
- reuses the handed-off `bwrap`/`busybox` roots instead of scanning the store,
- reports those exact roots in proof lines.

## Approach

### Hidden handoff arguments

Extend `crunch self-build` with hidden arguments:

- `--bootstrap-bwrap-path <path>`
- `--bootstrap-busybox-path <path>`

These are proof-internal plumbing, not general user-facing workflow.

### Validation rules

The handed-off paths must:

- be executable,
- have the expected file name (`bwrap` or `busybox`),
- live under the active self-build `--store` directory.

If a handed-off path is invalid, later-stage self-build fails immediately.

### Reuse instead of rediscovery

When handed-off paths are present, later-stage step `[2/4]` reuses those exact
roots for `bwrap` and `busybox` rather than rebuilding or rediscovering them by
suffix scan.

## Non-Goals

- changing stage0 host-prerequisite behavior
- changing unrelated bootstrap source URLs or mirrors
- changing non-proof `crunch build` tool resolution semantics
