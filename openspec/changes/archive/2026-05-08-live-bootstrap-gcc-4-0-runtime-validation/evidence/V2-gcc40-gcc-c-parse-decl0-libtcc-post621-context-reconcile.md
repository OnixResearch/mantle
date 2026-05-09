# V2 gcc40 c-parse decl0 libtcc post-621 context reconciliation

Task-ID: V2 focused TinyCC `libtcc.c` line-621 paired-cleanup replay reconciliation for the GCC 4.0 c-parse `decl0` predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Command

The replay must run under a `/crunch/store` bind because the restored TinyCC and musl outputs embed logical Crunch store paths:

```sh
BASH=$(command -v bash)
nix shell nixpkgs#bubblewrap -c bwrap \
  --dev /dev --proc /proc --tmpfs /tmp --dir /crunch \
  --bind "$PWD/.crunch-drain/post621-restored-store" /crunch/store \
  --ro-bind /nix /nix --ro-bind /usr /usr \
  --ro-bind /run/current-system /run/current-system --ro-bind /home /home \
  --chdir "$PWD" "$BASH" \
  openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-post621-context-reconcile-20260509/run-local-reconcile-logical.sh \
  > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-libtcc-post621-context-reconcile-20260509/focused-local-reconcile-logical.txt 2>&1
```

Exit status: 0.

## Evidence

- Run directory: `evidence/gcc40-cparse-decl0-libtcc-post621-context-reconcile-20260509/`
- Script: `evidence/gcc40-cparse-decl0-libtcc-post621-context-reconcile-20260509/run-local-reconcile-logical.sh`
- Focused log: `evidence/gcc40-cparse-decl0-libtcc-post621-context-reconcile-20260509/focused-local-reconcile-logical.txt`
- Replay inputs:
  - TinyCC: `.crunch-drain/post621-restored-store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2/bin/tcc`
  - musl headers: `.crunch-drain/post621-restored-store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl/include`
  - TinyCC source: `.crunch-drain/post621-restored-store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src`

## Focused results

| Variant | common | one_source |
| --- | ---: | ---: |
| `prefix621_before_native387` | 139 | 139 |
| `prefix621_after_native387` | 0 | 0 |

## Interpretation

The apparent contradiction between the archived line-621 pass and the later local replay was caused by replay context, not nondeterminism and not post-621 declaration identity.

The archived derivation ran the libtcc paired-cleanup prefix-growth probes **after** the earlier `native387_disabled` mutation to `tccgen.c`. A local replay that omitted that prerequisite made every line-621 prefix crash with `rc=139`. Replaying the same line-621 paired-cleanup source after applying the native387-disabled mutation restores the archived result: `rc=0` under both common and `ONE_SOURCE=1` flags.

So the current boundary remains: `tccgen.c` native x87 compile context must stay disabled for this predecessor TinyCC before interpreting later `libtcc.c` post-open or post-621 probes. No direct `decl0` runtime success is claimed here.
