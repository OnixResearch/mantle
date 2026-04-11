## ADDED Requirements

### Requirement: Explicit bootstrap trust inventory

The repo MUST document the externally trusted inputs and host prerequisites for
all supported bootstrap entry points.

At minimum, the inventory MUST cover:
- `crunch bootstrap`
- `crunch bootstrap --fetch`
- `crunch self-build`
- `./scripts/prove-self-hosting.sh`

For each path, the repo MUST name:
- externally trusted binaries, tarballs, or seeds,
- required host tools,
- what build or proof evidence the repo currently has,
- which stronger bootstrap claim is still not proven.

#### Scenario: Contributor inspects fetch bootstrap trust roots

- GIVEN a contributor wants to understand `crunch bootstrap --fetch`
- WHEN they read the bootstrap docs
- THEN they can see that the flow currently trusts a pinned fetched musl-gcc
  seed
- AND they can see which host tools are still required around that flow
- AND they can see that this is not yet the same as a full-source bootstrap

#### Scenario: Contributor inspects self-hosting proof trust roots

- GIVEN a contributor wants to understand the checked-in self-hosting proof
- WHEN they read the bootstrap docs
- THEN they can see that stage0 starts from a checkout-built crunch binary
- AND they can see which host prerequisites the proof helper assumes
- AND they can see what the proof demonstrates today

### Requirement: Bootstrap best-practice checklist is explicit

The repo MUST translate the relevant Bootstrappable Builds best practices into
an explicit checklist for crunch.

At minimum, that checklist MUST cover:
- whether crunch has an alternative build path for the build system itself,
- whether bootstrap binary or tarball provenance is clearly labeled,
- whether bootstrap binaries are reproducible from source end-to-end,
- whether bootstrap traceability or self-hosting checks are automated.

Each checklist line MUST be backed by concrete repo evidence and marked with a
current status such as yes, partial, or not yet.

#### Scenario: Contributor asks whether crunch follows bootstrap best practices

- GIVEN a contributor reads the bootstrap docs or spec
- WHEN they ask whether crunch follows the Bootstrappable Builds guidance
- THEN they can find a checklist with concrete statuses
- AND each status cites repo evidence or a named gap
- AND partial compliance is not presented as full compliance

### Requirement: Bootstrap claims stay separated by maturity level

The repo MUST distinguish between different bootstrap claims instead of using a
single overloaded label.

At minimum, the docs and bootstrap spec MUST separate:
- seed-assisted bootstrap,
- self-hosting proof,
- full-source bootstrap,
- reproducible or independently reproduced release evidence.

The repo MUST NOT describe a weaker property as if it proved a stronger one.

#### Scenario: Self-hosting proof is not presented as full-source bootstrap

- GIVEN the checked-in stage1 -> stage2 self-hosting proof exists
- WHEN a contributor reads the README or bootstrap spec
- THEN the repo states that the proof demonstrates a working self-rebuild path
- AND it does not claim that crunch is already fully bootstrapped from a tiny
  audited source seed
- AND it does not claim bit-for-bit reproducible release outputs unless such
  evidence is present

### Requirement: Bootstrap roadmap names remaining trust-reduction work

The repo MUST track the major milestones and blockers between the current
bootstrap story and a stronger full-source bootstrap story.

That roadmap MUST identify at least:
- remaining fetched or opaque bootstrap seeds,
- remaining host-tool prerequisites,
- proof gaps between self-hosting and reproducible release evidence,
- the next trust-reduction milestones the project intends to pursue.

#### Scenario: Contributor can see what comes after today’s proof

- GIVEN a contributor reads the bootstrap roadmap
- WHEN they compare current proof coverage to future goals
- THEN they can identify which milestones are already complete
- AND they can identify which trust anchors still remain
- AND they can see that reducing those anchors is tracked work rather than
  implied completion
