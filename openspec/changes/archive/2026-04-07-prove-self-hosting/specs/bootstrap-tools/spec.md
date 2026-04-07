## MODIFIED Requirements

### Requirement: Self-build uses crunch-built tools

The self-build pipeline MUST expose enough evidence to show whether it used
crunch-built sandbox tools or host fallbacks.

The first stage MAY fall back to a host `bwrap` when no crunch-built `bwrap`
exists yet. Once the proof store contains crunch-built `bwrap` and `busybox`,
a later self-build stage MUST report that it selected those crunch-built tools.

#### Scenario: Proof records crunch-built tool selection

- GIVEN a proof store that already contains crunch-built `*-bwrap` and
  `*-busybox` outputs
- WHEN the stage1 binary runs the second self-build stage
- THEN the proof output records that bwrap source was `crunch-built`
- AND it records the selected busybox path used for
  `SNIX_BUILD_SANDBOX_SHELL`

#### Scenario: First-stage host fallback stays visible

- GIVEN no crunch-built `bwrap` exists yet
- WHEN the first self-build stage runs
- THEN the output records that host fallback was used
- AND the proof workflow treats that fallback as acceptable only for the first
  stage
