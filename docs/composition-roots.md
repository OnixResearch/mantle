# Experimental composition roots

Mantle can combine exact castore directory roots into one immutable castore root. This command is experimental. It accepts concrete object facts only.

## Commands

Validate a request without store access:

```text
mantle --json store composition plan --from request.json
```

Realize complete roots from the selected local store:

```text
mantle --json store composition realize \
  --from request.json \
  --receipt-out receipt.json
```

The realization command writes the receipt only after input loading, merge planning, directory persistence, and root recheck succeed.

## Request schema

The request has two records:

- `plan` uses schema `mantle-composition-plan-v1`.
- `realization_policy` uses schema `mantle-composition-policy-v1`.

Each binding contains an exact castore directory digest and size, a normalized relative mount, and an optional input label. An empty mount means the composition root. Mantle derives `binding_ref` from the root and mount. It discards labels before it emits the normalized plan.

A collision decision contains an exact logical path and the winning derived `binding_ref`. Binding order is not precedence.

The policy contains named bounds for bindings, decisions, entries, depth, path bytes, one file, and total file bytes. It also states whether symlinks are admitted. The policy has a separate `realization_policy_ref`.

See `fixtures/composition-roots/equivalent-a.json` and `equivalent-b.json`. They have different order and labels but produce the same `plan_ref`.

## Merge model

Directories merge recursively. Identical leaves deduplicate. Different leaves, or a directory and leaf at one path, require one applicable collision decision. Mantle rejects these cases:

- missing decisions;
- duplicate decisions;
- stale decisions;
- a winner that does not contribute at the path;
- duplicate bindings;
- absolute, parent, empty, drive-prefixed, or non-normalized paths.

Symlink targets remain opaque bytes. Mantle does not follow them.

## Receipt

A successful `mantle-composition-receipt-v1` binds:

- `plan_ref`;
- `realization_policy_ref`;
- merge-policy version;
- sorted input root references;
- directory-union, leaf-deduplication, and explicit-replacement outcomes;
- measured limit usage;
- the resulting castore root reference;
- unsupported metadata classes;
- explicit non-claims.

The castore root is the filesystem identity. Mantle does not add a competing tree digest.

## Metadata support

| Fact | Status |
|---|---|
| directory names and structure | supported for UTF-8 names |
| file content digest and size | supported |
| executable bit | supported |
| symlink target bytes | supported and not followed |
| ownership and ACLs | unsupported |
| capabilities and security xattrs | unsupported |
| hard-link identity | unsupported |
| device nodes | unsupported |

A consumer that needs an unsupported class must not promote this experimental root to a production system root.

## Boundary

The request cannot contain package constraints, provider alternatives, machine roles, inventory, activation, deployment, or rollback policy. Unknown fields fail during bounded JSON decoding.

Mantle does not require OnixOS, Kamacite, Preserves, OCI, or Nix-compatible paths for composition identity or realization. Later adapters must use separate Cairn changes and map into this same semantic model.

A receipt proves bounded composition identity, linkage, validation, and realization facts only. It does not prove ABI compatibility, dependency completeness, runtime correctness, bootability, authorization, deployment success, or release eligibility.
