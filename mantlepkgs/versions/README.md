# Mantlepkgs version resolution

This producer workflow resolves reported package versions to exact Nixpkgs revisions before ordinary Mantlepkgs generation.

The no-Nix consumer path does not use these commands. Consumers use the generated Mantlepkgs catalogs and receipts.

Index, resolve, and recheck plan bounded input, producer, publication, and read-back effects
before their first capability call. The producer and publication adapters report their actual
result; each command independently re-reads its published artifacts and classifies the effect
observations before printing a success or blocked receipt.

## Files

- `contracts.ncl` defines the typed review contracts.
- `fixtures/` contains positive and negative contract fixtures.
- `pilot/` declares the two-version `x86_64-linux` pilot.

## Produce an index

Select one exact Nix executable. Then observe the declared revision cohort.

```console
NIX=$(readlink -f "$(command -v nix)")
mantle mantlepkgs version index \
  --cohort mantlepkgs/versions/pilot/cohort.ncl \
  --nix-program "$NIX" \
  --output-root target/mantlepkgs-version-pilot/index
```

The command writes the sealed cohort, complete observation set, and compact index through one no-replace directory publication.

Each observation has `success`, `unavailable`, or `failed` status. The producer does not parse derivation names for versions.

## Resolve requests

Resolve the checked request intent against one explicit saved index.

```console
mantle mantlepkgs version resolve \
  --index target/mantlepkgs-version-pilot/index/indexes/x86_64-linux.json \
  --policy mantlepkgs/versions/pilot/policy.ncl \
  --requests mantlepkgs/versions/pilot/requests.ncl \
  --output-root target/mantlepkgs-version-pilot/resolve
```

The shell binds blank policy placeholders to the explicit index. It binds the request set to the resulting sealed policy.

The pure core writes one receipt for each request. It groups resolved requests by exact revision.

A blocked request still gets a receipt. The command returns a nonzero status when any request is blocked.

## Recheck and emit manifests

Materialize each selected source. Then recheck its hash and package version.

```console
mantle mantlepkgs version recheck \
  --plan target/mantlepkgs-version-pilot/resolve/production-plan.json \
  --template-manifest mantlepkgs/live-cohort/manifest.ncl \
  --nix-program "$NIX" \
  --output-root target/mantlepkgs-version-pilot/recheck
```

The source-tree BLAKE3 hash includes paths, file bytes, executable bits, directories, and symlink text. It does not follow symlinks.

A successful group gets `manifest.json`, `contracts.ncl`, and `manifest.ncl`. Existing Mantlepkgs generation commands accept `manifest.ncl`.

A failed recheck publishes evidence without success manifests. The command does not select a different revision.

## Claim boundary

A successful receipt proves the recorded resolution and recheck under exact inputs. It does not prove package correctness or compatibility.

It also does not prove cache retention, evaluator parity, reproducibility, deployment safety, or release eligibility.
