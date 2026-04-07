## Context

crunch already has a real `self-build` command and a bootstrap chain that can
reach `busybox`, `bwrap`, `rust`, and finally `crunch`. The missing piece is a
repeatable proof artifact.

Right now the repo makes a stronger claim than it proves:

- `README.md` documents self-build as working.
- `src/self_build.rs` contains the runtime logic.
- `examples/crunch.ncl` still describes self-hosting as a placeholder.
- There is no end-to-end stage1 -> stage2 proof in `tests/`.

The proof has to answer a narrow question well: can a crunch-built `crunch`
binary rebuild crunch again, while selecting crunch-built sandbox tools when
those tools already exist?

## Goals / Non-Goals

**Goals:**
- One repeatable in-tree proof of self-hosting
- Explicit evidence for which binary drove each stage
- Explicit evidence for whether bwrap and busybox came from crunch outputs or
  the host
- A proof that forces a fresh final crunch build for stage2, rather than
  accepting a cache hit on the final binary

**Non-Goals:**
- Bit-for-bit fixed-point proof between stage1 and stage2
- Rebuilding the whole bootstrap chain from nothing for every proof run
- Removing host prerequisites like `git`, `cargo`, `tar`, or `xz`
- Running this proof on every normal test invocation

## Decisions

### 1. Use a slow proof runner, not a new primary CLI command

**Choice:** Implement the proof as a dedicated slow test/helper that runs on
demand, then document that command in the README.

**Rationale:** This is a regression-proofing workflow, not a new operator
surface. Keeping it as a test/helper avoids growing the supported CLI just to
host a long-running developer check.

**Alternative rejected:** Add a new top-level `crunch prove-self-hosting`
command. It would be convenient, but it widens the shipped CLI for a workflow
that is mostly for contributors and release validation.

### 2. Prove stage2 with the stage1 binary and a fresh final-output build

**Choice:** The proof run uses the checkout binary to build stage1, then invokes
`stage1/bin/crunch` to run the second stage. Before stage2, the proof invalidates
only the prior final `*-crunch` output and uses a fresh state directory.

**Rationale:** That preserves the stage1-built toolchain outputs needed for
self-hosting while preventing a false pass from a cached final crunch binary.
The question is whether stage1 can carry the final build forward, not whether we
can afford to rebuild every bootstrap tool from scratch on each proof run.

**Alternative rejected:** Use a completely fresh output store for stage2. That
would also drop the stage1-built bwrap and busybox, which are exactly the tools
we need stage2 to pick up.

**Implementation:**
- stage0 uses the checkout-built binary to run `crunch self-build`
- proof runner finds `stage1/bin/crunch`
- proof runner removes the prior `*-crunch` output and resets stage2 state
- proof runner invokes `stage1/bin/crunch self-build`
- proof runner verifies the resulting stage2 binary with `--help` or
  `--version`

### 3. Emit stable proof evidence from self-build

**Choice:** Self-build should print or return stable stage facts: which crunch
binary ran, where the output binary landed, which bwrap source was selected, and
which busybox path was used for `SNIX_BUILD_SANDBOX_SHELL`.

**Rationale:** The current log text is aimed at humans. A proof runner needs
stable markers so it can fail for the right reason instead of grepping brittle
free-form output.

**Alternative rejected:** Infer everything from filesystem scans and existing
stderr text. That would work for a first pass, but it is harder to maintain and
harder to diagnose when the proof fails.

### 4. Treat host-tool fallback as stage-specific

**Choice:** The proof allows host fallback for the first stage when there is no
crunch-built bwrap yet. Once stage1 has produced `*-bwrap` and `*-busybox`, the
second stage must report that it selected those crunch-built tools.

**Rationale:** That matches the real bootstrap story. The first run has a
chicken-and-egg problem. The second run is where self-hosting should show up.

**Alternative rejected:** Demand zero host fallback from stage0. That would
contradict the current documented bootstrap model.

## Risks / Trade-offs

**[Long runtime]** The proof is expensive. Keep it out of the default test path
and document expected runtime.

**[False cache passes]** If stage2 reuses the stage1 final crunch output, the
proof says very little. Delete the final `*-crunch` output and use a fresh state
for the second stage.

**[Brittle log parsing]** Free-form stderr is easy to break. Stable proof
markers or a structured summary keep the assertions simple.

**[Overclaiming]** A successful stage2 build proves self-hosting for the current
workflow. It does not prove reproducible fixed points, offline bootstrap, or
freedom from every host tool.
