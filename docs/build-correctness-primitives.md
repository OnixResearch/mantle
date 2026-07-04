# Build correctness primitives

Mantle's strong build-correctness evidence is expressed with frontend-neutral
records. Frontends such as Onix may lower evaluated semantics into these records,
but Mantle treats frontend refs as opaque data and does not interpret module
roles, tags, options, providers, or Nickel contracts.

## Records

- `mantle-action-spec-v1` declares action kind, platform, toolchain object refs,
  input object refs, argument and environment digests, output declarations,
  sandbox policy, network policy, expected reference policy, optional frontend
  spec refs, and optional Nickel evaluation provenance.
- `mantle-nickel-eval-receipt-v1` binds a `.ncl` root source ref, transitive
  import/source refs, import-path policy, evaluator identity, export/build-IR
  shape, and output digest.
- `mantle-object-manifest-v1` admits files, directories, symlinks, generated
  payloads, and redacted secret descriptors as `mantle-object://blake3/<digest>`
  refs. Path exports are only views and are not object identity.
- `mantle-reference-scan-v1` records discovered runtime/output references and
  rejects undeclared refs, forbidden refs, path traversal, duplicate conflicting
  views, and plaintext secret bytes.
- `mantle-action-receipt-v1` binds action refs, input/toolchain/output object
  refs, sandbox and network policy, reference-scan evidence, producer identity,
  signatures, execution status, and build-or-reuse reason.

## Network policy

Ordinary derivation actions are offline by default. Mantle only grants network
access to declared fixed-output fetcher actions whose URL, hash mode, expected
digest, and bounded retry policy are part of the action boundary. A foreign or
compatibility action that requests build-time network access must use an explicit
sandbox capability with action name, capability, policy basis, and audit class;
policy-denied or undeclared exceptions fail closed before strong evidence is
emitted.

## Reuse policy

Reuse or substitution can satisfy a strong claim only when the candidate receipt
matches the requested action ref, output object refs, sandbox policy, network
policy, producer policy, and required signatures. Mismatches fail closed instead
of falling back to path-only identity.

## Non-goals

A successful action receipt only claims that produced objects match the declared
action and receipt policy. It does not claim compiler correctness,
source-to-binary reproducibility, full bootstrap correctness, frontend module
correctness, deploy success, or physical-target determinism.

## Secret handling

Receipts and object manifests may carry redacted secret descriptors or encrypted
object refs. They must not serialize decrypted secret bytes or inline plaintext
secret content.
