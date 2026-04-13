## Context

crunch now has three nearby but distinct bootstrap properties:

1. **Seed-assisted bootstrap**: `crunch bootstrap --fetch` can fetch an
   initial static seed without Nix runtime commands.
2. **Self-hosting proof**: the checked-in proof demonstrates that a
   crunch-built stage1 binary can rebuild crunch and reach a stage1==stage2
   fixed point.
3. **First bootstrap without Nix on the host**: not yet proven.

The current code still carries several stage0 shortcuts that are acceptable for
self-hosting but not for the stronger claim:

- the proof helper may use `nix-build` to realize `pkgsStatic.busybox`
- tool resolution searches NixOS/Nix-specific locations
- stage0 source preparation shells out to host `git`, `tar`, `cp`, `sh`, and
  `cargo vendor`
- the current proof bundle records fixed-point evidence, not the absence of
  Nix commands from the first-bootstrap path

This change does not eliminate all of those dependencies immediately. It gives
crunch a precise target so future implementation work can reduce them without
moving the goalposts.

## Goals / Non-Goals

**Goals**
- Define what the repo means by a “Nix-free first bootstrap.”
- Separate that claim from the already-proven self-hosting fixed point.
- Require explicit accounting for stage0 prerequisites and hidden fallbacks.
- Break the remaining work into staged, reviewable implementation milestones.

**Non-Goals**
- Eliminate every stage0 host prerequisite in this change.
- Replace the current self-hosting proof.
- Guarantee a tiny audited seed in this change.
- Prove bit-for-bit release reproducibility in this change.

## Decisions

### 1. Treat hidden Nix fallback as a spec violation for first bootstrap

**Choice:** the stricter first-bootstrap path must not invoke `nix-build`,
`nix-store`, `nix-shell`, or `nix develop`, even as contingency behavior.

**Rationale:** a command that silently realizes a missing seed with Nix is not
Nix-free bootstrap. It is a Nix-assisted recovery path.

**Alternative:** allow hidden Nix fallback as long as the happy path avoids it.

**Why not:** that makes the strongest claim depend on local machine accident.
If a contributor forgets to preinstall one seed tool, the helper quietly stops
proving what it says it proves.

### 2. Classify stage0 inputs into explicit buckets

**Choice:** every stage0 dependency must be classified as one of:

- **host prerequisite** — installed by the operator and listed explicitly
- **pinned fetched artifact** — downloaded by crunch or the helper with a
  checked hash
- **crunch-built output** — produced earlier in the bootstrap chain

Anything outside those buckets counts as an undeclared trust edge.

**Rationale:** this keeps the bootstrap inventory honest and reviewable.

**Alternative:** keep a looser “whatever the helper finds on this host” model.

**Why not:** host autodiscovery across NixOS/Nix paths hides real trust roots
and makes first-bootstrap evidence non-portable.

### 3. Separate self-hosting proof from non-Nix-host proof

**Choice:** retain the current stage0->stage1->stage2 self-hosting proof, but
require a separate proof mode for the stronger claim that first bootstrap does
not need Nix commands on the host.

**Rationale:** the current proof is valuable and should not be overloaded. A
stronger proof needs stricter setup: Nix commands absent from `PATH`, explicit
seed inventory, and failure on hidden fallback.

**Alternative:** stretch the meaning of the existing proof bundle.

**Why not:** a fixed-point proof and a non-Nix-host proof answer different
questions. Merging them would blur review boundaries again.

### 4. Stage the remaining work

**Choice:** track the remaining implementation work as four milestones:

1. remove Nix fallback from seed provisioning and helper scripts
2. make stage0 source staging independent of host shell+git/tar glue
3. remove host `cargo vendor` dependency from stage0 source preparation
4. add a repeatable proof run with Nix commands intentionally unavailable

**Rationale:** each milestone removes one distinct class of trust edge and can
be implemented/tested independently.

**Alternative:** open one giant “fully bootstrapped” task.

**Why not:** that hides progress and makes review too fuzzy.

## Risks / Trade-offs

**Risk: overconstraining operator convenience** -> Mitigation: allow documented
host prerequisites, but require them to be explicit instead of discovered via
hidden Nix fallback.

**Risk: too much documentation without implementation** -> Mitigation: keep the
spec tightly coupled to concrete follow-up milestones and proof requirements.

**Risk: confusion with existing maturity model** -> Mitigation: frame this
change as the next step after the maturity model: now that the repo names the
claim boundary, it also needs a concrete plan to cross it.
