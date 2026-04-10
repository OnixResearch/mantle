## MODIFIED Requirements

### Requirement: Self-build uses crunch-built tools

The self-build pipeline MUST keep using an executable sandbox shell while it
bootstraps the crunch-built busybox and bwrap roots. A host-prerequisite
preflight alone is not enough; the full proof path MUST still get through the
stage0 `busybox.ncl` bootstrap without falling back to an unusable `/bin/sh`
inside bwrap.

#### Scenario: Stage0 busybox bootstrap keeps a usable shell

- GIVEN `./scripts/prove-self-hosting.sh --check` succeeds on the host
- AND the self-hosting proof starts stage0 from a checkout-built `crunch`
- WHEN stage0 builds `bootstrap/busybox.ncl`
- THEN the sandbox shell path used by bwrap is executable inside the sandbox
- AND the build does not fail with `bwrap: execvp /bin/sh: No such file or directory`
