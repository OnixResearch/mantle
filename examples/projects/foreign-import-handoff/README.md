# Foreign derivation handoff

This workflow validates and plans checked Guix-like and Nix-like artifacts without invoking either foreign frontend. Run from this directory.

## Inspect the typed workflow

```sh
mantle eval workflow.ncl
```

## Validate and plan

```sh
mantle --json foreign-import validate \
  --graph fixtures/guix-hello.graph.json \
  --package-index fixtures/guix-hello.index.json \
  --policy fixtures/policy.json > guix-validate.json

mantle --json foreign-import plan \
  --graph fixtures/nix-hello.graph.json \
  --package-index fixtures/nix-hello.index.json \
  --policy fixtures/policy.json \
  --package hello \
  --system x86_64-linux > nix-plan.json
```

Inspect `accepted`, `receipt.raw_graph_digest`, and `plan.plan_identity`. Also
inspect `plan.roots`, `plan.native_units`, `plan.exact_path_maps`, and
`plan.source_requirements`. The `plan.non_claims` field keeps realization claims
out of this step.

To demonstrate frontend independence, resolve the Mantle binary. Then run with
an empty `PATH`:

```sh
mantle_bin=$(command -v mantle)
empty_path=$(mktemp -d /tmp/mantle-foreign-path.XXXXXX)
PATH="$empty_path" "$mantle_bin" --json foreign-import plan \
  --graph fixtures/nix-hello.graph.json \
  --package-index fixtures/nix-hello.index.json \
  --policy fixtures/policy.json \
  --package hello \
  --system x86_64-linux
```

## Negative path

The policy has no trusted cache scopes, so this must fail with `untrusted-cache-hint`:

```sh
mantle --json foreign-import validate \
  --graph fixtures/untrusted-cache.graph.json \
  --package-index fixtures/guix-hello.index.json \
  --policy fixtures/policy.json
```

From the repository root, the focused rail also covers malformed input, stale receipts, embedded source rewrites, and undeclared sandbox capabilities:

```sh
nix develop -c cargo test -p mantle --test foreign_import_cli
```

Accepted validation proves only that the supplied artifacts satisfy the import
policy. An executable plan proves bounded compilation only. It does not prove
source availability, scheduler execution, store admission, realization, output
trust, package correctness, reproducibility, or release readiness.
