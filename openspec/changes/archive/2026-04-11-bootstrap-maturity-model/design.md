## Context

crunch now has several real bootstrap-related behaviors:

- `crunch bootstrap` imports seed paths from an existing Nix installation.
- `crunch bootstrap --fetch` imports a pinned musl-gcc tarball without Nix.
- `crunch self-build` builds crunch with a crunch-built busybox and bwrap once
  the bootstrap-tool stage has completed.
- `./scripts/prove-self-hosting.sh` proves that a crunch-built stage1 binary
  can drive a fresh stage2 self-build.

Those are meaningful milestones, but they are not the same guarantee.
The references make that separation explicit:

- Bootstrappable Builds treats opaque binary seeds as a quantity to be reduced
  and audited.
- Guix explains its bootstrap path as a graph rooted in a tiny auditable seed,
  and names the remaining large bootstrap driver as unfinished work.
- StageX separates “bootstrapped”, “reproducible”, and signing-policy claims
  instead of collapsing them into one marketing label.

crunch should adopt that style of precision. The current tree has evidence for
self-hosting progress, but not yet for a full-source bootstrap or reproducible,
independently reproduced releases.

## Goals / Non-Goals

**Goals**
- Make bootstrap claims precise and reviewable.
- Document current trust anchors and host prerequisites.
- Give contributors one roadmap from today’s proof to a stronger bootstrap
  story.
- Prevent docs/specs from overclaiming properties the repo does not yet prove.

**Non-Goals**
- Eliminate the fetched musl-gcc seed in this change.
- Remove host Rust/tooling prerequisites in this change.
- Prove bit-for-bit reproducibility in this change.
- Add signing/quorum policy machinery in this change.

## Decisions

### 1. Model bootstrap as distinct maturity levels

**Choice:** Define separate claim levels instead of using “self-hosting” as a
catch-all term.

Suggested distinctions:
- **Seed-assisted bootstrap**: crunch can start from declared external seeds
  and build later stages.
- **Self-hosting proof**: a crunch-built binary can rebuild crunch.
- **Full-source bootstrap**: the trusted root is reduced to small, explicitly
  audited source/bootstrap seeds.
- **Reproducible release**: independent rebuilds produce the same final
  artifact and can be compared or signed.

**Rationale:** This matches the distinctions made in the references and keeps
future review honest.

**Alternative:** Keep one broad “self-hosting/bootstrap” label.

**Why not:** It hides major differences between “works from a fetched toolchain
seed” and “full-source bootstrap from a tiny audited root”.

### 2. Inventory trust anchors by command path

**Choice:** Track trusted inputs and prerequisites per user-visible path:
`bootstrap`, `bootstrap --fetch`, `self-build`, and the self-hosting proof.

Each path should identify:
- external binaries or tarballs it trusts,
- host tools it assumes,
- what evidence exists today,
- what stronger claim is still blocked.

**Rationale:** The current repo has multiple entry points, and each has a
slightly different trust boundary.

**Alternative:** Keep one generic bootstrap note.

**Why not:** Contributors need to know which path still depends on Nix, which
one depends on fetched tarballs, and which one already uses crunch-built
sandbox tools.

### 3. Add an explicit best-practice checklist

**Choice:** Turn the Bootstrappable Builds best-practices page into a concrete
repo checklist instead of leaving it as background reading.

Current checklist:

| Best-practice item | Current status in crunch | Evidence / gap |
|---|---|---|
| Build-system writers should provide an alternative way to build the build system | **Mostly yes** | `cargo build --release` builds `crunch` without needing a prior `crunch`; `crunch self-build` then provides a self-hosting path. |
| Distros should clearly label where bootstrap binaries came from and how they were produced | **Partly** | `bootstrap --fetch` pins the musl-gcc URL and hash in `src/bootstrap.rs`, but the repo does not yet present one consolidated trust inventory for every bootstrap path. |
| Distros should let users reproduce bootstrap binaries | **Not yet, end-to-end** | The repo proves stage1 -> stage2 self-hosting, but it does not yet reproduce the fetched musl-gcc seed from source all the way down. |
| Distros should automate reproducibility / traceability of bootstrap binaries | **Partly** | `./scripts/prove-self-hosting.sh` automates the self-hosting proof, but the README already says this is not bit-for-bit reproducibility or freedom from host tools. |

**Rationale:** This keeps the OpenSpec grounded. We are not just inventing a
new taxonomy; we are measuring the repo against a published checklist.

**Alternative:** Mention the best-practices page only as inspiration.

**Why not:** That would lose the concrete “are we doing this or not?” framing
that motivated the follow-up.

### 4. Keep the first change documentation-first

**Choice:** Start with spec + docs + roadmap, not with a new implementation
surface.

**Rationale:** The immediate gap is conceptual clarity. The code already does
more than the docs say in some places and less than a reader may infer in
others.

**Alternative:** Add a new `crunch bootstrap status` command immediately.

**Why not:** That is a separate implementation choice. First we should agree on
what information must exist and how the repo should classify bootstrap claims.

## Risks / Trade-offs

**Too much abstraction** -> Mitigation: keep the maturity model tied to
concrete repo commands and proof artifacts.

**Docs drift from code** -> Mitigation: require the inventory to cite checked-in
commands/helpers and review it against current proof behavior.

**Spec overreach** -> Mitigation: make this change about explicit claims and
roadmap only, not about promising full-source bootstrap immediately.
