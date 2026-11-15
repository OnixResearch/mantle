# 0154. Rebind captured workspace descendants to their declared root

Status: Accepted

## Context

The current source-built Rust recipe has 57 translation invocations whose
`OUT_DIR` word names a descendant `/tmp/mantle-rust/.../out` as its Workspace
mount. `bind_tmp_step` declares only the captured `/tmp` workspace rooted in
its `generated-tmp` output. The unchanged invocation decoder requires an exact
workspace-map key and rejects those words.

Adding an alias for each descendant is not a safe repair: the consumer rejects
overlapping writable workspaces. Turning the operand into literal path data,
relaxing exact lookup, or stripping pathname components would instead lose its
explicit ownership or change the original compiler environment.

## Decision Drivers

- Preserve the exact original virtual pathname and captured generated contents.
- Use the one existing writable `/tmp` Tree and its existing seed identity.
- Keep exact scope lookup, overlap checks, and lexical path admission unchanged.
- Reject traversal and unknown roots rather than guessing a writable mapping.
- Normalize at the adapter that owns the captured-workspace invocation.

## Decision

Before serializing a captured command's canonical invocation, `bind_tmp_step`
rebases Workspace operands whose mount is a strict descendant of `/tmp` onto
that exact declared root. The validated suffix becomes the beginning of the
relative member, followed by any validated existing member. A word for
`/tmp/mantle-rust/script/out` plus member `generated` therefore still resolves
to `/tmp/mantle-rust/script/out/generated`, now through the declared `/tmp`
workspace.

Only typed Workspace parts in the static arguments/environment are changed.
The original pathname is preserved; literal/compiler data and other authority
kinds are untouched. Existing `/tmp` words remain unchanged. Other roots are
not aliased and remain subject to consumer rejection. Both the descendant and
member must pass the existing relative-path contract before rebinding.

The transformation reuses the old mount string for the relative member rather
than cloning complete operands. No consumer validation, overlapping mapping,
source output, source/provider pin or runtime fallback is introduced.

## Consequences

- The captured mrustc translation adapter owns this boundary. Its current
  consumer is minicargo library translation after original build-script output
  capture; the durable capability is consistent typed ownership of generated
  paths under one captured workspace.
- A regression emits the real canonical invocation through `bind_tmp_step` and
  submits those bytes to the production decoder. It checks exact descendant
  pathname preservation and rejection of unknown workspaces and traversal.
  The pre-fix execution and subsequent actual verification are retained under
  the active fixed-point change, not inferred from static source text.
- Existing source-script/proc-macro workspace conventions are not globally
  rewritten. This repair applies at the captured `/tmp` boundary that declares
  and seeds the actual writable Tree.
- Source and generated program identities change. Keep `9300895d…` immutable
  and unlaunched; a replacement source/runner must be frozen/qualified, its
  source programs regenerated, and its current data reviewed before launch.
- This authoring repair or its decoder test does not establish compiler
  correctness, protected provider execution, fixed-point equality,
  reproducibility, signed native-row acceptance or release eligibility.
