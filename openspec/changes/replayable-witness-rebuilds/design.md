## Context

The repo already covers the edges around a witnessed self-hosting release:

- publisher-side proof creation is standardized through
  `./scripts/prove-self-hosting.sh`,
- release evidence packages the source archive, proof bundle, and published
  binaries,
- `crunch release witness-export` hands a public request directory to a second
  environment, and
- `crunch attest witness-import` and `crunch attest release-verify` can fold a
  matching witness back into the publisher's verification directory.

The missing step is the witness-side rebuild itself. Today the witness has to:

1. inspect the exported request directory,
2. choose a rebuild recipe manually,
3. decide where scratch and outputs should live,
4. produce rebuilt binaries, and only then
5. call `crunch attest witness-create` by hand.

That makes the strongest currently-supported trust story partially informal.
The repo says it wants independent rebuild evidence for release outputs, but it
still lacks a checked-in witness rebuild rail that another operator can replay
without reverse-engineering the expected workflow.

## Goals / Non-Goals

**Goals**
- Introduce one checked-in witness rebuild entry point for exported request
  directories.
- Keep request parsing and rebuild planning fail-closed and deterministic.
- Reuse the existing release-evidence, release-attestation, witness-create,
  witness-import, and release-verify surfaces.
- Capture witness-side rebuild evidence before sidecars are imported.
- Document and test the full publisher -> witness-runner -> publisher rail.

**Non-Goals**
- Reduce the seed provider or remove stage0 host-tool prerequisites.
- Add hosted witness discovery, transparency logs, or online policy
  distribution.
- Generalize the first implementation to arbitrary workflow identities beyond
  the checked-in self-hosting proof path.
- Claim that a same-host integration test proves real-world witness
  independence.

## Decisions

### 1. Split witness replay into a pure rebuild-plan core and a thin shell

**Choice:** derive a pure `WitnessRebuildPlan` from the exported request
artifacts, then keep filesystem mutation, subprocess execution, and scratch-dir
setup in a thin shell.

**Rationale:** this fits the repo's functional-core / imperative-shell rule and
lets the fragile parts fail before rebuild starts. The pure core can validate:
- request schema and layout,
- release identifier consistency,
- supported workflow identity,
- expected published outputs, and
- deterministic output and audit locations.

**Implementation:** place the plan-building logic in a Rust core module that
returns owned data only. The command shell in `src/release_cmd.rs` consumes
that plan, runs the rebuild, and forwards the rebuilt outputs into the existing
witness-attestation path.

### 2. Make `crunch release witness-rebuild` the public witness-side command

**Choice:** add `crunch release witness-rebuild <request-dir>` as the canonical
machine-readable entry point.

**Rationale:** the exported request directory is rooted in release evidence, so
this belongs under `crunch release` rather than `crunch attest`. The command
will own request validation, rebuild execution, and witness-side artifact
layout. Existing `crunch attest witness-create` stays as the lower-level escape
hatch and internal reuse point.

**Alternative:** ask operators to keep calling `crunch attest witness-create`
after a manual rebuild.

**Why not:** that leaves the most important witness-side step undocumented and
non-replayable.

### 3. Keep `./scripts/rebuild-witness-request.sh` as the host-preflight shell

**Choice:** add `./scripts/rebuild-witness-request.sh` as the checked-in wrapper
for `crunch release witness-rebuild`.

**Rationale:** the witness-side rebuild has the same operational risks as the
publisher proof path: toolchain discovery, OpenSSL/pkg-config visibility,
`bwrap`, static sandbox shell selection, and scratch capacity. A checked-in
helper keeps the operator workflow honest without pushing host-environment logic
into the pure core.

**Implementation:** the helper will:
- discover `bwrap` before invoking the CLI,
- resolve a real static busybox-backed `SNIX_BUILD_SANDBOX_SHELL`, absolutize
  it, and reject placeholder or missing shells,
- treat the exported request directory as immutable after validation and derive
  the witness scratch root from `CRUNCH_WITNESS_SCRATCH_DIR` when set, else
  from the sibling path `<request-dir>.work/`,
- rewrite `TMPDIR` and `CARGO_TARGET_DIR` under that scratch root,
- provide a `--check` mode for prerequisite preflight only, and
- state explicitly that `--check` success is not evidence of a successful
  witness rebuild.

**Alternative:** require every witness environment to discover the right shell
and toolchain setup manually.

**Why not:** that would keep the workflow replayable only for people who already
know the repo's current proof environment rules.

### 4. Derive the replay recipe from one canonical workflow-identity pair

**Choice:** the runner will consume the recorded workflow identity already bound
into the request's release attestation and proof bundle, and the first slice
will support exactly one pair:
- `workflow_command = "./scripts/prove-self-hosting.sh"`
- `workflow_version = "crunch-self-hosting-proof-v2"`

**Rationale:** the witness should replay the publisher's declared workflow, not
invent a different build recipe and only compare digests at the end. Binding the
runner to one exact `(command, version)` pair also gives a clean fail-closed
boundary for unsupported future workflows.

**Implementation:** the pure plan builder accepts only that exact ordered pair
and returns a hard error naming the unsupported `workflow_command` /
`workflow_version` values for anything else.

### 5. Persist witness-side rebuild evidence before import

**Choice:** `crunch release witness-rebuild` writes both witness sidecars and a
rebuild audit directory under deterministic paths in the witness work area.

**Rationale:** the publisher already has `witness-import` for the canonical
sidecars. The missing piece is durable witness-side evidence that explains how
those sidecars were produced.

**Implementation:** the audit directory will include a deterministic `meta.json`
summary naming at least:
- request schema and layout version,
- release identifier,
- `workflow_command` and `workflow_version`,
- selected scratch root, rewritten `TMPDIR`, and rewritten `CARGO_TARGET_DIR`,
- command line used to launch the rebuild,
- started and finished timestamps captured around the subprocess rather than
  post-hoc,
- rebuilt output names and inherited BLAKE3 digests from the existing
  release-evidence comparison path, and
- relative paths to the produced witness sidecars.

Large logs or artifacts stay referenced by digest and path rather than copied
blindly into the summary.

**Alternative:** write only the witness sidecars.

**Why not:** that would preserve the final agreement artifact while still
leaving the witness-side rebuild opaque.

## Interaction sequence

```text
publisher
  verified release bundle
    -> crunch release attest
    -> crunch attest policy-init
    -> crunch release witness-export
    -> request directory

witness
  request directory
    -> ./scripts/rebuild-witness-request.sh (preflight wrapper)
    -> crunch release witness-rebuild
    -> verify bundled release evidence
    -> derive WitnessRebuildPlan
    -> replay supported self-hosting workflow
    -> rebuilt outputs + witness sidecars + rebuild audit directory

publisher
  returned witness sidecars
    -> crunch attest witness-import
    -> crunch attest release-verify
    -> technical class + policy status + final class
```

## Verification strategy

- unit tests for the pure request-to-plan core: supported workflow, unsupported
  workflow/layout, release-id mismatch, and output-list validation.
- CLI integration tests for `crunch release witness-rebuild`: happy path,
  unsupported workflow/layout rejection, rebuilt-output mismatch rejection, and
  proof that helper `--check` stays preflight-only.
- end-to-end workflow test: `release attest -> policy-init -> witness-export ->
  witness-rebuild -> witness-import -> release-verify`.
- docs verification that README and `docs/operator-workflows.md` name the
  checked-in witness rebuild step and keep claims bounded to external witness
  agreement under configured policy.

## Risks / Trade-offs

**The first runner only supports one workflow identity.**
That is intentional. The repo currently has one checked-in self-hosting proof
workflow worth replaying. Wider workflow polymorphism can follow after the
first honest rail exists.

**The helper script adds another public workflow entry point.**
That is acceptable because the proof path already uses this pattern: a script
for host preflight and a Rust command for the core logic.

**Witness-side rebuild evidence can grow large.**
The audit directory should stay summary-oriented and reference large artifacts
by deterministic path and digest instead of copying everything again.
