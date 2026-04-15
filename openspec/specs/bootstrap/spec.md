## MODIFIED Requirements

### Requirement: Strict proof mode rejects host fallback after bootstrap tools exist

The checked-in self-hosting proof entry point MUST support a strict hermetic
mode for later proof stages.

In strict mode:

- stage0 MAY still use declared host prerequisites needed for first bootstrap,
- once crunch-built `busybox` and `bwrap` roots exist, stage1 and later stages
  MUST bind to those exact outputs,
- stage1 and later stages MUST carry the exact stage0-produced `bwrap` and
  `busybox` paths into subsequent `crunch self-build` invocations instead of
  re-discovering later-stage tool roots by scanning the output store,
- stage1 and later stages MUST fail if sandbox-tool resolution falls back to
  host discovery,
- existing strict staged-source fallback rejection MUST remain in force, but
  this change does not alter that behavior,
- the proof summary MUST record the selected hermeticity mode and fallback
  events using explicit `self-build-proof: fallback-event=...` markers,
- a successful zero-fallback later stage MUST emit `self-build-proof: fallback-event=none`.

#### Scenario: Stage2 rejects host fallback tool discovery

- GIVEN stage0 already produced crunch-built `busybox` and `bwrap` roots
- AND the proof is running in strict mode
- WHEN stage1 starts the later self-build stage
- THEN stage2 uses those exact crunch-built tool roots
- AND the proof fails if tool selection falls back to host discovery

#### Scenario: Stage2 ignores stale sibling tool outputs

- GIVEN stage0 already produced exact `bwrap` and `busybox` roots
- AND stale sibling `*-bwrap` or `*-busybox` entries also exist in the store
- WHEN stage1 starts the later strict self-build stage
- THEN stage2 still uses the exact stage0-produced tool roots
- AND stage2 reports those exact stage0-produced tool roots in its proof lines
- AND it does not switch to a stale sibling found by store scanning

#### Scenario: Successful strict proof reports zero later-stage fallbacks

- GIVEN a successful strict self-hosting proof run
- WHEN the proof summary is written
- THEN it records hermeticity mode `strict`
- AND it reports the exact crunch-built tool roots used by later stages
- AND it reports that no later-stage host fallback events occurred
