## MODIFIED Requirements

### Requirement: Repeatable self-hosting proof

The repo MUST provide a repeatable proof that a crunch-built `crunch` binary
can rebuild crunch from the same staged source tree.

The proof MUST include at least two stages:
- stage0: a checkout-built binary runs `crunch self-build` and produces
  stage1
- stage1: the produced stage1 binary runs `crunch self-build` again and
  produces stage2

The proof MUST record which binary drove each stage, which staged source tree
was used, and where each resulting binary landed.

#### Scenario: Stage1 drives stage2 from the recorded staged source

- GIVEN a checkout-built `crunch` binary and a writable proof workspace
- WHEN the self-hosting proof runs
- THEN stage0 produces a working stage1 `bin/crunch`
- AND stage0 records the staged `*-crunch-src` path it used
- AND the proof invokes that stage1 binary for the second stage
- AND stage2 reuses that same staged source path instead of restaging from the checkout
- AND stage2 produces a working `bin/crunch`

#### Scenario: Stage2 is isolated from checkout staging fallback

- GIVEN the proof has already recorded a staged `*-crunch-src` path from stage0
- WHEN the stage1 binary runs the second stage
- THEN the proof launches stage2 from outside the repo root
- AND it clears `PATH` before the stage2 command starts
- AND the stage2 run still succeeds by reusing the recorded staged source path
