## ADDED Requirements

### Requirement: Strict proof mode rejects host fallback after bootstrap tools exist

The checked-in self-hosting proof entry point MUST support a strict hermetic
mode for later proof stages.

In strict mode:

- stage0 MAY still use declared host prerequisites needed for first bootstrap,
- once crunch-built `busybox` and `bwrap` roots exist, stage1 and later stages
  MUST bind to those exact outputs,
- stage1 and later stages MUST fail if sandbox-tool or staged-source resolution
  falls back to host discovery,
- the proof summary MUST record the selected hermeticity mode and any fallback
  events that occurred.

#### Scenario: Stage2 rejects host fallback tool discovery

- GIVEN stage0 already produced crunch-built `busybox` and `bwrap` roots
- AND the proof is running in strict mode
- WHEN stage1 starts the later self-build stage
- THEN stage2 uses those exact crunch-built tool roots
- AND the proof fails if tool selection falls back to host discovery

#### Scenario: Successful strict proof reports zero later-stage fallbacks

- GIVEN a successful strict self-hosting proof run
- WHEN the proof summary is written
- THEN it records hermeticity mode `strict`
- AND it reports the exact crunch-built tool roots used by later stages
- AND it reports that no later-stage host fallback events occurred
