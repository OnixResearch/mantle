## MODIFIED Requirements

### Requirement: Self-build uses crunch-built tools

The self-build pipeline MUST use the crunch-bootstrapped busybox and bwrap
instead of externally-provided binaries, after the initial bootstrap.

Once step 2 has exported `bwrap` and `busybox` as root outputs, step 3 MUST
bind the final self-build derivation to those exact exported store entries.
It MUST NOT rediscover those tools later by scanning the proof store for the
first matching sibling output.

#### Scenario: Proof records exact crunch-built tool selection

- GIVEN a proof store that already contains crunch-built `*-bwrap` and
  `*-busybox` outputs from the bootstrap-tool step
- WHEN the stage1 binary runs the second self-build stage
- THEN the proof output records that bwrap source was `crunch-built`
- AND it records the selected busybox path used for
  `SNIX_BUILD_SANDBOX_SHELL`
- AND the stage2 build log reports those same exact crunch-built tool paths

#### Scenario: Final derivation uses exported tool roots directly

- GIVEN step 2 built root outputs `X-bwrap` and `Y-busybox`
- WHEN step 3 generates the final self-build derivation
- THEN the derivation input list includes `/.../X-bwrap` and `/.../Y-busybox`
- AND the shell script uses those exact paths for `bwrap` and `busybox`
- AND the shell script does not scan `$NIX_STORE/*-bwrap` or `$NIX_STORE/*-busybox`
