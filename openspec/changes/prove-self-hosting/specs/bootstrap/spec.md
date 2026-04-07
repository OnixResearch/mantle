## ADDED Requirements

### Requirement: Repeatable self-hosting proof

The repo MUST provide a repeatable proof that a crunch-built `crunch` binary
can rebuild crunch from the same source tree.

The proof MUST include at least two stages:
- stage0: a checkout-built binary runs `crunch self-build` and produces
  stage1
- stage1: the produced stage1 binary runs `crunch self-build` again and
  produces stage2

The proof MUST record which binary drove each stage and where each resulting
binary landed.

#### Scenario: Stage1 drives stage2

- GIVEN a checkout-built `crunch` binary and a writable proof workspace
- WHEN the self-hosting proof runs
- THEN stage0 produces a working stage1 `bin/crunch`
- AND the proof invokes that stage1 binary for the second stage
- AND stage2 produces a working `bin/crunch`

#### Scenario: Produced binary is executable

- GIVEN a completed self-hosting proof run
- WHEN the proof checks the produced stage2 binary
- THEN `crunch --help` or `crunch --version` succeeds
- AND the proof reports the stage2 binary path

### Requirement: Stage2 must rebuild the final crunch output

The self-hosting proof MUST force a fresh final crunch build for stage2.

The proof MAY reuse previously built bootstrap tools such as `bwrap`,
`busybox`, `rust`, or `gcc`, but it MUST NOT accept a cache hit for the final
`*-crunch` output as evidence of self-hosting.

#### Scenario: Final binary cache hit is rejected as proof

- GIVEN stage1 already wrote a `*-crunch` output in the proof store
- WHEN stage2 starts
- THEN the proof invalidates or removes the prior final `*-crunch` output
- AND stage2 performs a fresh final crunch build
- AND the proof fails if stage2 only reports a final-binary cache hit

### Requirement: Documented self-hosting workflow

The repo MUST document how to run the self-hosting proof, what prerequisites it
needs, and what a successful run proves.

#### Scenario: Contributor follows the documented proof path

- GIVEN a contributor wants to verify self-hosting locally
- WHEN they follow the documented proof instructions
- THEN they have the exact command to run
- AND they know the required host prerequisites
- AND they know that the proof demonstrates a working stage1 -> stage2 rebuild,
  not a reproducible fixed point
