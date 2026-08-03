# Cache-imported Nixpkgs tool use

This workflow imports one signed Nixpkgs `mksh` closure. A separate Mantle derivation then uses that exact shell as its builder.

The imported shell keeps `cache-imported` provenance. Only the consumer output is Mantle-built.

See [`evidence.md`](evidence.md) for the recorded validation facts.

Run all commands from this directory.

## Prerequisites

- Use Linux with Bubblewrap support.
- Enter the repository Nix development shell.
- Build `mantle` from this checkout.
- Allow producer access to Nixpkgs and `cache.nixos.org`.

## Fixed inputs

| Input | Value |
|---|---|
| Nixpkgs revision | `dfd9566f82a6e1d55c30f861879186440614696e` |
| Package | `mksh` |
| Root derivation | `/nix/store/ibzx4zxyq58c1d6cv41505y152c9incd-mksh-59c.drv` |
| Selected output | `/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c` |
| Builder | `/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c/bin/mksh` |

`mksh` has one output in this revision. The cache-only route currently requires one selected output root.

Static BusyBox has separate `out` and `debug` outputs in this revision. Use it after the cache-only route supports explicit output selection.

## Prepare the producer artifacts

Keep Nix in this producer step only.

```sh
repo_root=$(cd ../../.. && pwd)
example_root="$repo_root/examples/projects/nixpkgs-tool-use"
scratch=/tmp/mantle-nixpkgs-tool-use
nix_ref=github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e
root_drv=/nix/store/ibzx4zxyq58c1d6cv41505y152c9incd-mksh-59c.drv
mksh_out=/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c
cache_key='cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY='

rm -rf "$scratch"
mkdir -p "$scratch/no-nix-path"

test "$(nix path-info --derivation "$nix_ref#mksh")" = "$root_drv"
test "$(nix eval --raw "$nix_ref#mksh.outPath")" = "$mksh_out"
nix path-info --store https://cache.nixos.org "$mksh_out"
nix derivation show --recursive "$nix_ref#mksh" > "$scratch/derivation-json.json"

mantle --json foreign-import produce-nix \
  --derivation-json "$scratch/derivation-json.json" \
  --root-derivation "$root_drv" \
  --package mksh \
  --system x86_64-linux \
  --producer-identity 'github:NixOS/nixpkgs#mksh' \
  --producer-revision dfd9566f82a6e1d55c30f861879186440614696e \
  --cache-url "https://cache.nixos.org?trusted_public_keys[0]=$cache_key" \
  --out-dir "$scratch/artifacts" > "$scratch/produce-report.json"
```

Resolve the consumer tools before you remove Nix commands from `PATH`:

```sh
mantle_bin=$(command -v mantle)
bwrap_bin=$(command -v bwrap)
ln -s "$bwrap_bin" "$scratch/no-nix-path/bwrap"

if PATH="$scratch/no-nix-path" command -v nix >/dev/null 2>&1; then
  exit 1
fi
if PATH="$scratch/no-nix-path" command -v nix-store >/dev/null 2>&1; then
  exit 1
fi
```

## Import the signed closure

These commands use a `PATH` that contains no Nix command.

```sh
PATH="$scratch/no-nix-path" "$mantle_bin" --json --nix-compat \
  foreign-import plan \
  --graph "$scratch/artifacts/nixpkgs.graph.json" \
  --package-index "$scratch/artifacts/nixpkgs.index.json" \
  --policy "$repo_root/config/foreign-cache-closure/generated/preserve-nix.json" \
  --package mksh \
  --system x86_64-linux \
  --execution-profile "$repo_root/config/foreign-execution-profiles/generated/nix.json" \
  --plan-out "$scratch/plan.json" \
  --receipt-out "$scratch/import-receipt.json" > "$scratch/plan-report.json"

PATH="$scratch/no-nix-path" "$mantle_bin" --json --nix-compat \
  foreign-import prepare-sources \
  --plan "$scratch/plan.json" \
  --out "$scratch/source-bundle.json" > "$scratch/source-report.json"

PATH="$scratch/no-nix-path" "$mantle_bin" --json \
  --state-dir "$scratch/state" \
  --store "$scratch/output" \
  --nix-compat \
  foreign-import realize \
  --plan "$scratch/plan.json" \
  --import-receipt "$scratch/import-receipt.json" \
  --source-bundle "$scratch/source-bundle.json" \
  --source-bundle-blake3 8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777 \
  --execution-profile "$repo_root/config/foreign-execution-profiles/generated/nix.json" \
  --cache-closure-policy "$repo_root/config/foreign-cache-closure/generated/default.json" \
  --receipt-out "$scratch/realization-receipt.json" \
  --substitute > "$scratch/realization-report.json"
```

The realization receipt must report `status: complete`. It must also report `cache-only-preserve-v1` as the route.

## Build with the imported shell

For the strongest host-fallback evidence, confirm that the exact logical path is absent from the host store:

```sh
test ! -e "$mksh_out"
```

Then run the strict consumer build:

```sh
PATH="$scratch/no-nix-path" CRUNCH_NO_FUSE=1 "$mantle_bin" --json \
  --state-dir "$scratch/state" \
  --store "$scratch/output" \
  --nix-compat \
  build "$example_root/consumer.ncl" \
  --strict-hermetic \
  --no-substitute > "$scratch/consumer-build-report.json"
```

The build report must contain these facts:

- `hermeticity_mode` is `strict`.
- `built_total` is `1`.
- `failed_total` is `0`.
- `hermeticity_audit_events` is empty.
- The recorded search-path entries are empty.

The flat output must contain:

```text
builder=/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c/bin/mksh
provenance=cache-imported
workflow=nixpkgs-tool-use
execution=imported-builder
```

## Missing-state negative case

A fresh state has no imported `mksh` PathInfo. This command must fail before builder execution:

```sh
mkdir -p "$scratch/negative-output"

PATH="$scratch/no-nix-path" CRUNCH_NO_FUSE=1 "$mantle_bin" --json \
  --state-dir "$scratch/negative-state" \
  --store "$scratch/negative-output" \
  --nix-compat \
  build "$example_root/consumer.ncl" \
  --strict-hermetic \
  --no-substitute > "$scratch/negative-build-report.json"
```

The negative report must use `missing-source-input`. It must name the exact `mksh` store path.

## Cache-admission negative cases

Run the focused cache-closure tests from the repository root:

```sh
nix develop -c cargo test -p crunch-store http_closure_untrusted_root_fails_before_nar_download
nix develop -c cargo test -p crunch-store http_closure_changed_dependency_nar_keeps_closure_absent
```

The first test supplies a signer that the trust policy does not accept. It fails before NAR transfer and does not add PathInfo.

The second test changes the dependency NAR bytes. It rejects the closure and keeps both dependency and root PathInfo absent.

## Claim boundary

This workflow proves that Mantle used one signed, cache-imported Nixpkgs tool for one recorded build. It also proves that the consumer output was built by Mantle.

It does not prove that Mantle built `mksh`. It does not prove Nix evaluator parity, package correctness, reproducibility, or release eligibility.
