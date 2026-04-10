## Phase 1: Source binding

- [x] Add internal self-build plumbing to reuse an exact staged source tree between proof stages
- [x] Emit the staged-source proof marker from `SelfBuildReport`
- [x] Reject reused staged-source trees whose contents no longer match their staged store name
- [x] Normalize reused staged-source paths so a stage2 cwd change does not reinterpret them

## Phase 2: Exact bootstrap-tool binding

- [x] Thread the exact exported `bwrap` root output into the generated self-build derivation
- [x] Thread the exact exported `busybox` root output into the generated self-build derivation
- [x] Remove stage-3 store-glob rediscovery of `bwrap` and `busybox`

## Phase 3: Proof hardening

- [x] Run stage2 from outside the repo with `PATH` cleared
- [x] Assert that stage2 reused the exact staged source path from stage0
- [x] Assert that the stage2 build log used the exact reported crunch-built `bwrap` and `busybox` paths
