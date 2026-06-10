## Context

Nix's correctness story is built from multiple layers: derivations describe declared actions, store objects are content-addressed or input-addressed with known references, closures are scanned, builders run under policy, substitutes are verified, and receipts explain why outputs are reused. Mantle currently has pieces of this for selected workflows, but it does not expose one frontend-neutral correctness primitive set that OnixOS can depend on for no-Nix-runtime VM artifacts.

## Decisions

### 1. Define action specs instead of Nix derivations

**Choice:** Mantle will define `mantle-action-spec-v1` as a generic derivation-like record. It binds action kind, platform, toolchain refs, input refs, args digest, environment digest, output declarations, sandbox policy, network policy, expected references, and frontend artifact spec refs when present.

**Rationale:** This captures the useful invariant--declared inputs produce declared outputs--without importing Nix syntax, evaluation, or store-path identity.

### 2. Use BLAKE3 refs for action and object identity

**Choice:** Action refs use `mantle-action://blake3/<digest>` and object refs use `mantle-object://blake3/<digest>` unless an accepted spec defines a narrower family ref. Canonicalization must be deterministic and domain-separated by schema and object kind.

**Rationale:** Mantle already uses BLAKE3-oriented evidence. Explicit ref families prevent action ids, object ids, and frontend closure roots from being accidentally interchangeable.

### 3. CAS admission is distinct from paths

**Choice:** Mantle may expose path views for execution and export, but object identity comes from CAS refs and manifests. The CAS admission layer records object kind, content digest, size, executable/mode metadata when modeled, symlink targets, directory children, and redacted secret descriptors.

**Rationale:** Nix paths are convenient but path identity is not portable to Mantle's no-Nix-runtime goal.

### 4. Hermetic policy is required before strong claims

**Choice:** Strong action-correctness claims require a declared sandbox/network policy and an execution report proving that the policy was enforced. If Mantle cannot enforce the requested policy on the current host, it must fail closed or mark the result as fixture-only/narrower evidence.

**Rationale:** Declared inputs are only meaningful when ambient host reads and networks are controlled.

### 5. Reference scanning is a first-class stage

**Choice:** Mantle will scan output artifacts or consume verified scan reports to prove runtime references are declared and allowed by policy. Unsupported scanners, forbidden refs, or stale scan roots fail closed for action-correct claims.

**Rationale:** Nix's closure/reference checks are a core correctness invariant. Mantle needs a frontend-neutral equivalent.

### 6. Reuse and substitution require receipt equivalence

**Choice:** Output reuse or external substitution may be accepted only when action ref, input refs, toolchain refs, sandbox policy, reference-scan policy, output object refs, and producer policy match the requested trust policy. Missing signatures or mismatched policy must reject strong reuse claims.

**Rationale:** Reuse is where many correctness claims can become unsound. Receipts must explain why an output is accepted without rerunning the action.

### 7. Keep frontend semantics outside Mantle

**Choice:** Frontends such as Onix may attach spec-admission data, expected refs, and artifact kinds. Mantle validates generic specs and receipts; it does not interpret roles, tags, settings, providers, NixOS options, or Nickel contracts.

**Rationale:** This preserves Mantle's build-tool boundary and lets multiple frontends use the same correctness primitives.

## Data Model Sketch

```text
MantleActionSpec {
  schema = "mantle-action-spec-v1"
  action_ref
  action_kind
  platform
  toolchain_refs[]
  input_object_refs[]
  args_digest_blake3
  env_digest_blake3
  output_declarations[]
  sandbox_policy
  network_policy
  expected_reference_policy
  frontend_spec_refs[]
}

MantleActionReceipt {
  schema = "mantle-action-receipt-v1"
  action_ref
  execution_status
  produced_object_refs[]
  reference_scan_ref
  sandbox_report_ref
  producer_identity
  signature_refs[]
  reuse_or_build_reason
}
```

## Risks / Trade-offs

- **Scope growth:** Mitigated by keeping primitives generic and deferring frontend semantics.
- **Host portability:** Some platforms may lack a strong sandbox. Those must get explicit blockers or narrower fixture evidence.
- **Secret leakage:** CAS and reference manifests must support redacted descriptors and negative tests for plaintext secret bytes.
- **Substitution trust:** Signed receipt policy should be explicit and may start as local-only until remote trust roots are defined.

## Verification Strategy

- Pure canonicalization tests for action specs and object manifests.
- Positive CAS admission tests for files, directories, symlinks, and redacted descriptors.
- Negative CAS/reference tests for digest mismatch, path-only identity, duplicate conflicting views, path traversal, forbidden refs, and secret-byte leakage.
- Sandbox policy tests proving unsupported policies block strong claims.
- Reuse/substitution tests proving stale or unsigned receipts cannot satisfy a requested action.
- Boundary tests proving Onix/NixOS module-layer terms remain absent from Mantle core behavior.
