# Remote nix-derivation payload slice (2026-10-31)

Scope: add a remote build payload variant that a per-goal pool executor can
produce from what the Worker already holds, without the recursive Nickel
`CrunchDerivation` closure.

## Wire contract

- `RemoteConcreteBuildPayload::NixDerivation { drv_path, drv_json }` (tag
  `nix-derivation`) carries one canonical `nix_compat::derivation::Derivation`
  serialized as JSON, whose declared inputs are supplied by the ordinary upload
  closure.
- `plan_remote_nix_derivation_payload` validates the declared path against the
  store prefix, parses the derivation, recomputes its drv path from the declared
  name (stripping `.drv`) with `calculate_derivation_path_with_store_dir`, and
  reuses the existing `validate_derivation_plan_identity` checks for the
  expected outputs. The resulting plan source, command args, environment, and
  system come from that derivation, and its digest is the same ATerm BLAKE3 used
  for the closure-carrying variant.
- A receiver that rebuilds the graph locally still needs the recursive closure:
  `local_derivation_registry` rejects this variant with
  `remote-local-executor-nix-derivation-unsupported` instead of guessing.
- Workspace-policy detection reads the same `WORKSPACE_POLICY_ENV` from the nix
  derivation environment.
- Attenuated admission classifies the variant as `nix-derivation`; failure-debug
  redaction inspects its JSON like the other payloads.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle remote_build::tests::nix_derivation_payload` | 2 passed |
| `cargo test -p mantle --bin mantle remote_build::tests::` | 206 passed |
| strict Clippy (mantle targets), rustfmt on changed files | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

Fixtures plan the variant from the same derivation the Worker sees, assert the
declared/computed path agreement, name, digest length, argv, and system, then
reject a drifted declared path, malformed JSON, and the local rebuild executor.

## Not claimed

- No sender uses the variant yet: the pool executor and pipeline plumbing are
  the next step, so `build --builder-pool` still does not dispatch.
- A planned payload is not transfer, execution, or output-admission evidence.
