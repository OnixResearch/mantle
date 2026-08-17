# Live Nixpkgs hello command transcript

The paths below are relative to this evidence directory unless stated otherwise.
The trusted cache key is public cache metadata.

## Export with host Nix

```text
nix --version
nix flake metadata nixpkgs --json > nixpkgs-flake-metadata.json
nix derivation show --recursive nixpkgs#hello > derivation-json.json
```

The recursive export selected this root:

```text
/nix/store/0j6kpwz4dm9964pssxn1zf5vql05y3fl-hello-2.12.3.drv
```

## Produce concrete artifacts

```text
mantle --json foreign-import produce-nix \
  --derivation-json derivation-json.json \
  --root-derivation /nix/store/0j6kpwz4dm9964pssxn1zf5vql05y3fl-hello-2.12.3.drv \
  --package hello \
  --system x86_64-linux \
  --producer-identity live-nixpkgs#hello \
  --producer-revision registry-2026-08-02 \
  --cache-url 'https://cache.nixos.org?trusted_public_keys[0]=cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=' \
  --out-dir artifacts
```

## Plan without Nix

```text
PATH=/tmp/mantle-no-nix-path mantle --json --nix-compat foreign-import plan \
  --graph artifacts/nixpkgs.graph.json \
  --package-index artifacts/nixpkgs.index.json \
  --policy config/foreign-cache-closure/generated/preserve-nix.json \
  --package hello \
  --system x86_64-linux \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --plan-out plan.json \
  --receipt-out import-receipt.json
```

## Prepare the empty source bundle

```text
PATH=/tmp/mantle-no-nix-path mantle --json --nix-compat foreign-import prepare-sources \
  --plan plan.json \
  --out source-bundle.json
```

## Realize through trusted cache admission

```text
PATH=/tmp/mantle-no-nix-path mantle --json \
  --state-dir state \
  --store output \
  --nix-compat \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777 \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --cache-closure-policy config/foreign-cache-closure/generated/default.json \
  --receipt-out realization-receipt-preflight.json \
  --substitute
```

The same command used a new receipt path for the exact reuse run.

## Audit admitted castore facts

```text
PATH=/tmp/mantle-no-nix-path mantle --json \
  --state-dir state \
  --store output \
  --nix-compat \
  foreign-import audit \
  --plan plan.json \
  --realization-receipt realization-receipt-preflight.json \
  --policy config/foreign-provenance-audit/generated/default.json \
  --root nix:0j6kpwz4dm9964pssxn1zf5vql05y3fl-hello-2.12.3.drv \
  --out provenance-audit-final.json
```

## Hydrate a fresh state from the receipt

```text
PATH=/tmp/mantle-no-nix-path mantle \
  --state-dir state-pull \
  --store output-pull \
  --nix-compat \
  store pull \
  --from https://cache.nixos.org \
  --closure \
  --foreign-realization-receipt realization-receipt-preflight.json \
  --trusted-public-keys 'cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY='
```

All Mantle consumption commands used a `PATH` without `nix`.
The evidence bundle stores output facts, not cache NAR payloads or mutable state directories.
