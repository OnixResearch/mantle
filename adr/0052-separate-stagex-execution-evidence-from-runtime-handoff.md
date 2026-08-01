# ADR 0052: Separate StageX execution evidence from the runtime handoff

## Status

Accepted (2026-07-31)

## Context

The source-built fixed-point proof executes the protected StageX transition in a fresh output authority. The completed transition tree contains build scratch, negative-test fixtures, tool namespaces, and installed runtime outputs. Its scratch includes absolute and parent-relative symlinks that are valid only inside that execution tree.

Mantle then tried to adopt the complete execution tree as a source under the reserved StageX transition store path. Source admission correctly rejected those symlinks. Weakening source admission would let ambient or escaping links enter later builds. Rewriting the complete execution tree would also mutate the evidence that describes the protected transition.

The later native graph uses only eight declared StageX runtime output directories. It does not use the transition scratch or negative-test fixtures.

## Decision Drivers

- Preserve the complete protected execution tree and its audit without mutation.
- Keep source admission strict for absolute and parent-relative symlinks.
- Expose only the StageX outputs that the later native graph declares.
- Keep the reserved logical transition path fresh and deterministic.
- Bind final stage evidence to the complete execution tree, not to a reduced runtime projection.

## Decision

Mantle uses two distinct StageX directories in the source-built fixed-point proof.

The proof executes StageX in `stagex-transition-execution/`. This directory retains the complete transition report, protected-exec audit, build scratch, negative fixtures, and runtime outputs. StageX provider publication reads this complete tree. The final proof receipt records this directory as the StageX transition evidence output.

Mantle separately creates the reserved `*-mantle-stagex-transition` store path as a create-new runtime handoff. The handoff contains exactly these directories:

- `bash-full-stage/runtime/output`
- `coreutils-stage/runtime/output`
- `diffutils-stage/runtime/output`
- `gawk-stage/runtime/output`
- `grep-stage/runtime/output`
- `m4-stage/runtime/output`
- `make-stage/runtime/output`
- `sed-stage/runtime/output`

A deterministic handoff report records this fixed allowlist and its non-claim. Mantle validates every required source directory, runtime file, transition report, and protected-exec audit before it creates the handoff. It then uses bounded copies, validates the copied outputs, and routes the projection through ordinary strict source admission.

The complete execution tree remains unchanged. The runtime handoff does not contain binutils scratch, tool-namespace links, or transition evidence copies with run-specific paths.

The bound native source manifest can name the reserved transition and provider paths as empty virtual store-path records. Those records bind the current graph shape, but they are not acquisition payloads. The profile materializes the exact deduplicated union of fixed-fetch records from the native and StageX manifests. The proof constructs and imports the two StageX store paths before native offline preflight.

## Alternatives Considered

### Permit internal absolute symlinks during source admission

Rejected because admission would need ambient path interpretation and could accept links outside the declared source authority.

### Rewrite every transition symlink in place

Rejected because it mutates the completed execution evidence and still requires special handling for logical-prefix links.

### Copy the complete tree and replace links with regular files

Rejected because it promotes build scratch and negative fixtures into the runtime authority. It also creates a larger and less reviewable source surface.

## Consequences

- The fixed-point proof preserves full StageX execution evidence separately from the runtime source authority.
- Later native builds receive only declared StageX runtime outputs.
- Unsafe symlinks remain fail-closed at source admission.
- The StageX transition receipt hashes the preserved execution tree and `protected-exec-audit.json`.
- The runtime projection does not prove transition completion by itself.
- This decision does not prove native-provider admission, the Rust provider, the Mantle fixed point, compiler correctness, or release eligibility.
