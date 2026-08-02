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

## Realize a preserved Nix cache path

Use `preserve-cache-paths-v1` only with one unchanged `/nix/store` prefix.
The plan records the separate `cache-only-preserve-v1` route.
It maps each foreign output and source path to the same path.

Use this translation policy during planning:

```text
--policy config/foreign-cache-closure/generated/preserve-nix.json
```

Prepare the required empty source bundle without `--source`:

```text
mantle --json foreign-import prepare-sources \
  --plan plan.json \
  --out source-bundle.json
```

Then run cache-only realization:

```text
PATH=/path/without/nix mantle --json \
  --state-dir ./state \
  --store ./output \
  --store-prefix /nix/store \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --cache-closure-policy config/foreign-cache-closure/generated/default.json \
  --receipt-out realization-receipt.json \
  --substitute
```

The plan cache URL must bind each trusted public key.
Mantle supports one selected output root for this route.
It validates all bounded NARInfo metadata before it imports NAR content.
A metadata failure creates no output and no realization receipt.

After hydration, Mantle reopens the store without remote services.
The ordinary scheduler observes the selected output as already present.
The cache-only observer has no inputs, arguments, or executable builder fallback.
Units outside the runtime closure report `not-required-cache-only`.

## Export a translated GuixPkgs package

GuixPkgs is a producer boundary. Pin its flake before export. Record the flake
revision, `guix-metadata.json`, and locked `guix-transfer` revision.

Use producer-side Nix to export and realize the raw translated package:

```text
nix derivation show --recursive <guixpkgs-hello-unwrapped.drv> > derivation-json.json
nix build --no-link <pinned-guixpkgs-hello-unwrapped>
```

Generate a dedicated Nix cache signing key. Sign the complete runtime closure,
then copy it to a Nix-compatible cache:

```text
nix key generate-secret --key-name <exporter-name> > export-secret-key
nix key convert-secret-to-public < export-secret-key > export-public-key
nix store sign --key-file export-secret-key --recursive <hello-output>
nix copy --to 'file:///path/to/cache?compression=xz' <hello-output>
```

Do not retain the secret key in evidence or consumer state. Put the public key
in the graph's receipt-bound cache URL. Percent-encode `+` as `%2B` in query
values. Stop the producer boundary before Mantle planning starts.

Serve the exported cache over bounded HTTP. Then use the preserved-path flow
above with a `PATH` that contains no Nix or Guix command.

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

## Audit realized provenance

Audit selected roots after a complete realization:

```text
mantle --json \
  --state-dir ./state \
  --store ./output \
  foreign-import audit \
  --plan plan.json \
  --realization-receipt realization-receipt.json \
  --policy config/foreign-provenance-audit/generated/default.json \
  --root nix:hello \
  --out provenance-audit.json
```

The command loads the existing signing key. It verifies every selected PathInfo
before content scanning. It then reads directories and blobs from castore only.
It does not use exported host files as replacement content.

The scanner classifies data, ELF files, scripts, links, tar archives, newc initrds, gzip streams, zstd streams, and libtool archives.
It scans bounded gzip and zstd payloads after decompression.
It normalizes lexical `.` and `..` suffixes only when they stay inside one store root.
Unknown executable bytes fail closed.
Invalid store digests, missing targets, link escapes, malformed containers, and exhausted limits also fail closed.

A passing receipt reports `provenance-audited`. A failed audit reports
`realized` as its strongest state. Both results preserve the original
realization receipt and build-report identity.

Review `policy`, `observation_blake3`, `findings`, and `non_claims`. The audit
cannot prove package correctness, runtime behavior, reproducibility, bootability,
deployment safety, or release eligibility.

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
Mantle validates all closure metadata before it downloads content.
Receipt tampering, untrusted signatures, missing members, and exhausted limits fail closed.

## System boundary

This adapter owns package-graph source ingest and local store realization.
OnixOS still owns system intent, target assembly, activation, accounts, services, deployment, boot, and machine evidence.
A package realization receipt is not OnixOS deployment evidence.
