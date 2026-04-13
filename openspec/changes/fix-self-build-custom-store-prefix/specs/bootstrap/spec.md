## ADDED Requirements

### Requirement: Self-build respects the configured store prefix

The system MUST accept self-build source and bootstrap-tool store paths under
the configured logical store prefix, not just `/nix/store`.

#### Scenario: Stage3 accepts default `/crunch/store` inputs

- GIVEN `crunch self-build` runs with the default logical store prefix `/crunch/store`
- AND stage0 already produced staged source and bootstrap-tool outputs under that prefix
- WHEN stage3 evaluates and prepares the final crunch derivation
- THEN the source input path `/crunch/store/<hash>-crunch-src` is accepted
- AND the bootstrap tool paths `/crunch/store/<hash>-busybox` and `/crunch/store/<hash>-bwrap` are accepted
- AND the build does not fail only because a parser or contract hardcodes `/nix/store`
