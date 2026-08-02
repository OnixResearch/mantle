# Foreign derivation realization

Mantle can realize an admitted foreign derivation graph without running its foreign frontend.
The input must be a concrete, executable Mantle plan.
Mantle does not evaluate Nix, Nixpkgs, Guix, GuixPkgs, flakes, overlays, or package modules in this flow.

## Prepare a plan

Create the plan and import receipt together:

```text
mantle --json foreign-import plan \
  --graph tests/fixtures/foreign-import/realize-two-node.graph.json \
  --package-index tests/fixtures/foreign-import/realize-two-node.index.json \
  --policy tests/fixtures/foreign-import/realize-policy.json \
  --package two-node \
  --system x86_64-linux \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --plan-out plan.json \
  --receipt-out import-receipt.json
```

The execution profile is part of each derivation identity.
A profile change therefore changes target derivation paths.

## Prepare source records

Bind every non-derivation source requirement to an explicit local path:

```text
mantle --json foreign-import prepare-sources \
  --plan plan.json \
  --source foreign-builder=tests/fixtures/foreign-import/realize-two-node-builder.sh \
  --out source-bundle.json
```

The command accepts regular files and directory trees.
Directory records can contain bounded regular files and safe relative symbolic links.
The bundle records content, executable modes, symbolic-link targets, and BLAKE3 identities.

Copy the reported `manifest_blake3` value into the realization command.
Mantle rejects missing records, changed content, incomplete trees, and wrong modes before source ingest.
It does not read an ambient `/nix/store` or `/gnu/store` path to satisfy a requirement.

## Realize locally

Run the ordinary Mantle registry, scheduler, worker, and store path:

```text
mantle --json \
  --state-dir ./state \
  --store ./output \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --receipt-out realization-receipt.json \
  --offline \
  --no-substitute
```

Use `--root <foreign-derivation>` to select a subset of declared plan roots.
Use `--substitute` only when configured caches and trust keys can admit the required outputs.
Candidate source URLs keep plan order, and each attempt appears in the receipt.

Mantle rejects `--remote` for this adapter.
It also rejects stale receipts, unknown roots, profile mismatch, undeclared capabilities, and conflicting source content.
These checks occur before store mutation.

## Read the receipt

A successful run writes `mantle-foreign-realization-receipt-v1`.
The receipt binds these facts:

- executable-plan, import-receipt, source-bundle, and build-report BLAKE3 values;
- selected roots and exact target paths;
- execution-profile identities and digests;
- source PathInfo and output PathInfo observations;
- build, substitution, fetch, reuse, and failure dispositions;
- cache policy, ordered fetch attempts, strongest state, and non-claims.

A failed build writes `partial-failure` evidence when execution started.
Preflight rejection writes no realization receipt and does not create store state.

A complete receipt proves only the recorded local realization observations.
It does not prove package correctness, evaluator parity, reproducibility, provenance, release eligibility, or future frontend availability.

## Hydrate a receipt-selected cache closure

A complete receipt can supply one selected root to an HTTP closure pull:

```text
mantle store pull \
  --from https://cache.example.invalid/mantle \
  --closure \
  --foreign-realization-receipt realization-receipt.json \
  --trusted-public-keys 'mantle-cache-1:...'
```

Mantle verifies the receipt schema, status, BLAKE3 identity, and selected root before store mutation.
The cache must contain the receipt's target paths under the same logical store prefix.
The cache path still uses normal signature, NAR hash, store-prefix, and PathInfo checks.

## System boundary

This adapter owns package-graph source ingest and local store realization.
OnixOS still owns system intent, target assembly, activation, accounts, services, deployment, boot, and machine evidence.
A package realization receipt is not OnixOS deployment evidence.
