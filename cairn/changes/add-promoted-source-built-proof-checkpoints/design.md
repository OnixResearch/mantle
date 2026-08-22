# Design: Promoted source-built proof checkpoints

## Context

The six-stage proof already separates StageX transition, StageX provider publication, native-provider construction, Rust-provider construction, Mantle stage1, and Mantle stage2.

The existing dev cache keys provider reuse with the complete plan and source-authority digests. That is safe for development, but it invalidates providers when only later Mantle source changes.

A promoted checkpoint must preserve prior execution evidence. It must not convert a cache hit into a false current-execution claim.

## Decisions

### Decision: Bind each completed stage to its actual authority inputs

The functional core resolves each stage source, policy, and predecessor-output input from the six-stage plan. It computes one stage-authority BLAKE3 identity from those facts and the resource contract.

The provider checkpoint lookup key contains the union of source and policy inputs used by the first four stages. It excludes Mantle source, vendor inputs, and normalization policy because those facts first affect stage1.

A later Mantle-only edit can therefore reuse an unchanged provider checkpoint. A relevant StageX, native, Rust, policy, or resource change produces a hard miss.

### Decision: Publish one provider-closure checkpoint before stage1

The first implementation publishes after the full-source Rust provider and toolchain closure complete. The checkpoint contains four ordered stage records and seven required payload roles:

- StageX transition execution;
- StageX provider;
- native provider;
- Rust provider;
- native admission report;
- native build transcript; and
- toolchain closure.

Later work can add stage1 and stage2 checkpoints without changing the provider-checkpoint contract.

### Decision: Separate lookup, content, admission, and execution

The checkpoint lookup key discovers a candidate. It does not authorize reuse.

The shell remeasures every payload through no-follow bounded observation. The core then validates schema, promoted origin, stage order, source and policy authority, predecessor outputs, semantic provider identities, resource bounds, execution evidence, and payload identities.

A missing candidate selects the cold path. A present but malformed, partial, conflicting, or mismatched candidate fails closed.

### Decision: Restore into a fresh proof root

The shell copies the admitted payload into a new staging directory. It preserves regular files, directories, modes, and opaque symlink target bytes without following symlinks.

The shell rehashes restored payloads before it continues. It never executes from the checkpoint directory.

The closure payload contains absolute paths from its origin attempt. Mantle keeps those exact bytes under checkpoint-origin evidence. It derives a current closure from the restored providers, then requires identical member authority and provider-relative paths. Only the two fresh absolute provider roots may change. A relocation report binds both closure identities.

### Decision: Compose prior stage evidence without claiming current execution

A final receipt identifies each stage as `executed` or `restored`. A restored stage includes the checkpoint-manifest digest and the original promoted execution-evidence digest.

Planned-versus-observed reconciliation uses the original bound observations for restored stages. It does not require duplicate current-run execution events.

The completed proof claim is compositional: every stage was executed under admitted authority, but not necessarily in one process attempt.

### Decision: Keep cold proof behavior available

When no checkpoint store is selected, Mantle reads no checkpoint state and runs the existing cold path.

The current cold proof remains valid evidence for checkpoint production and as an independent negative control.

## Validation

Positive coverage must prove an exact promoted checkpoint can restore into a fresh directory and continue at stage1.

Negative coverage must reject dev origin, changed relevant source, changed policy, changed predecessor output, changed resource bounds, wrong semantic provider identity, missing payload, modified payload, forbidden events, and conflicting candidates.

A runtime cycle must record cold production, checkpoint publication, fresh-root adoption, a Mantle-only source change that preserves provider reuse, and a relevant provider-source change that forces rejection.

## Non-Claims

- A restored stage did not execute again in the adopting attempt.
- A payload digest alone does not prove provenance or authority.
- Checkpoint composition does not prove compiler correctness, independent rebuild agreement, or release reproducibility.
