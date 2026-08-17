# Live GuixPkgs hello command transcript

Paths below are relative to this evidence directory unless stated otherwise.
Producer commands may use Nix. Consumer commands use a `PATH` without Nix or Guix.

## Pin and export the producer graph

```text
nix --version > nix-version.txt
nix flake metadata --json github:fzakaria/guixpkgs > guixpkgs-flake-metadata.json
nix eval --raw github:fzakaria/guixpkgs#packages.x86_64-linux.hello.unwrapped.drvPath
nix eval --raw github:fzakaria/guixpkgs#packages.x86_64-linux.hello.unwrapped.outPath
nix derivation show --recursive /nix/store/nad8x3j7m2nz1x19678i2arv2g1g4k99-hello-2.12.2.drv > derivation-json.json
```

The producer built the pinned raw package with Guix sandbox-parity options:

```text
nix build --no-link --print-out-paths --impure \
  --expr 'let f = builtins.getFlake "github:fzakaria/guixpkgs/b3ec7b4f03c87a4d5fb67cc18e022567535e1795"; in f.packages.x86_64-linux.hello.unwrapped' \
  --option extra-substituters https://guixpkgs.cachix.org \
  --option extra-trusted-public-keys 'guixpkgs.cachix.org-1:rM4xwCs5NUy+FcCKkiWP/CmRaSVxxDPaKWZvM1bRopg=' \
  --option filter-syscalls false \
  --option sandbox-paths ''
```

The resulting program printed `Hello, world!`.

## Sign and export the runtime closure

```text
nix key generate-secret --key-name mantle-live-guixpkgs-export-1 > export-cache-secret-key
nix key convert-secret-to-public < export-cache-secret-key > export-cache-public-key
nix store sign --key-file export-cache-secret-key --recursive /nix/store/23afiym7dlgh1lxk1lyx6xrzplwnwhsk-hello-2.12.2
nix copy --to 'file:///tmp/mantle-live-guixpkgs-proof/export-cache?compression=xz' \
  /nix/store/23afiym7dlgh1lxk1lyx6xrzplwnwhsk-hello-2.12.2
```

The secret key remained under `/tmp` and is not retained. Evidence contains only
the public key and four NARInfo files. The proof used xz cache compression.

## Produce Mantle artifacts

The public-key query encodes each `+` as `%2B`.

```text
mantle --json foreign-import produce-nix \
  --derivation-json derivation-json.json \
  --root-derivation /nix/store/nad8x3j7m2nz1x19678i2arv2g1g4k99-hello-2.12.2.drv \
  --package hello \
  --system x86_64-linux \
  --producer-identity 'live-guixpkgs#hello.unwrapped' \
  --producer-revision 'guixpkgs-b3ec7b4f03c87a4d5fb67cc18e022567535e1795' \
  --cache-url 'http://127.0.0.1:18373?trusted_public_keys[0]=mantle-live-guixpkgs-export-1:KqV1aJoHpBe%2B5MZt1KLCjo%2BJjtFCeIVYdevTAbtOXpo=' \
  --out-dir artifacts
```

## Plan and prepare sources without frontends

```text
PATH=/tmp/mantle-live-guixpkgs-proof/no-nix-path mantle --json --nix-compat foreign-import plan \
  --graph artifacts/nixpkgs.graph.json \
  --package-index artifacts/nixpkgs.index.json \
  --policy config/foreign-cache-closure/generated/preserve-nix.json \
  --package hello \
  --system x86_64-linux \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --plan-out plan.json \
  --receipt-out import-receipt.json

PATH=/tmp/mantle-live-guixpkgs-proof/no-nix-path mantle --json --nix-compat foreign-import prepare-sources \
  --plan plan.json \
  --out source-bundle.json
```

## Realize and reuse without frontends

A copied BusyBox served the local cache. The consumer `PATH` contained only
copied Mantle and BusyBox binaries.

```text
PATH=/tmp/mantle-live-guixpkgs-proof/no-nix-path mantle --json \
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
  --receipt-out realization-receipt.json \
  --substitute
```

The same command used a new receipt path for the reuse run.

## Audit admitted castore facts

```text
PATH=/tmp/mantle-live-guixpkgs-proof/no-nix-path mantle --json \
  --state-dir state \
  --store output \
  --nix-compat \
  foreign-import audit \
  --plan plan.json \
  --realization-receipt realization-receipt.json \
  --policy config/foreign-provenance-audit/generated/default.json \
  --root nix:nad8x3j7m2nz1x19678i2arv2g1g4k99-hello-2.12.2.drv \
  --out provenance-audit.json
```

The command returned status 1 and retained one finding. This is the expected
fail-closed audit disposition for the recorded bytes.

## Hydrate a fresh state from the receipt

```text
PATH=/tmp/mantle-live-guixpkgs-proof/no-nix-path mantle \
  --state-dir state-pull \
  --store output-pull \
  --nix-compat \
  store pull \
  --from http://127.0.0.1:18373 \
  --closure \
  --foreign-realization-receipt realization-receipt.json \
  --trusted-public-keys 'mantle-live-guixpkgs-export-1:KqV1aJoHpBe+5MZt1KLCjo+JjtFCeIVYdevTAbtOXpo='
```

The fresh pull admitted four paths and reused zero paths.

## Negative commands

The negative matrix changed one trust fact at a time:

- pass the `cache.nixos.org` key instead of the exporter key;
- set `max_paths` to 1;
- replace the receipt digest with zeroes;
- remove the Bash NARInfo from a copy of the export cache.

Each command used a fresh state and output directory. Each returned status 3.
No negative preflight created a realization receipt or exported a store path.
