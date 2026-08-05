# fix producer backend

This directory pins and builds [psyclyx/fix](https://github.com/psyclyx/fix), a
from-scratch parallel evaluator for the Nix language written in Zig, and wires
it into Mantle as a swappable `nix` producer backend behind the
`nix-producer-v1` contract (`src/nix_producer.rs`).

## What lives here

| File | Role |
|---|---|
| `fix-src.ncl` | Fixed-output pin of the upstream fix source (rev `fd675c2e`, 0.3.0) |
| `zig-toolchain.ncl` | Fixed-output pin of the ziglang.org 0.16.0 binary tarball (fallback toolchain source) |
| `nixpkgs-toolchain.ncl` | Logical paths of the signed nixpkgs cache closures used for the build |
| `fix.ncl` | Sandboxed `zig build --release=fast` derivation |
| `compatibility-evidence.json` | Typed, pin-bound compatibility evidence record |

## Backend contract and selection

Every Nix producer backend satisfies the `nix-producer-v1` contract: bounded
expression inputs in, a `.drv` closure directory plus typed identity facts out.
Policy selects exactly one backend per run. Unknown kinds, missing binaries,
and unsupported systems fail closed before evaluation. No ambient fallback
substitutes another backend.

| Backend | Binary identity | Trust posture |
|---|---|---|
| `fix` | BLAKE3 digest of the Mantle-built binary | `pinned-mantle-built` |
| `host-nix` | Absolute path of `nix-instantiate` | `ambient-host` (not reproducible by Mantle) |

## Reproduce the build

```sh
# 1. Pull the signed toolchain closures (nixpkgs pin dfd9566f in nixpkgs-toolchain.ncl)
mantle --nix-compat --store <store> --state-dir <state> store pull \
  --from https://cache.nixos.org --closure \
  --trusted-public-keys cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= \
  <each path in nixpkgs-toolchain.ncl>

# 2. Build fix from the pinned source
mantle --nix-compat --store <store> --state-dir <state> build packages/fix/fix.ncl
```

The output attestation records runtime references to the toolchain inputs.
Both toolchain source classes (upstream tarball, signed nixpkgs cache) are
binary trust inputs, not source-built compilers.

## Run the producer

```sh
mantle --json foreign-import produce-backend \
  --backend fix \
  --backend-binary <store>/...-fix-0.3.0/bin/fix \
  --backend-version 0.3.0 \
  --expr '<nix expression>' \
  --package <name> \
  --work-dir <scratch> --out-dir <artifacts>
```

The backend runs under a bounded policy: wall-time deadline with owned
kill-and-reap teardown, capped stdout/stderr, and `.drv` closure byte and file
budgets. The closure is copied into an owned directory and admitted through
Mantle's own ATerm parsing. `fix instantiate` writes `.drv` files through a
reachable daemon; the adapter uses the daemon only as a store-write transport
and never runs realization commands (`fix build`, `fix run`, `fix switch`).

Artifacts are the standard `foreign-derivation-graph-v1` and
`foreign-package-index-v1` files. Consumption (validate, plan, realize) never
invokes a backend.

## Memory-limit caveat

`memory_bytes_max` applies `RLIMIT_AS`. It is disabled by default: the fix
parallel GC reserves over 64 GiB of virtual address space at startup, so an
address-space cap kills legitimate runs. A disabled limit is never reported as
enforcement. Honest RSS enforcement needs a cgroup mechanism and is deferred.

## Compatibility evidence

`compatibility-evidence.json` binds the evidence to exact pins. Upstream
reports 80,586 of 80,586 matching `.drv` paths on its pinned nixpkgs CI
universe at the pinned revision. Mantle has measured only a two-node
instantiation parity against host `nix-instantiate`. When any pin drifts,
`classify_evidence_freshness` in `src/nix_producer.rs` marks the record stale
for the new identity, and old counts must not be presented as covering it.

## Non-claims

- No claim of semantic equivalence between fix and Nix beyond the recorded pins.
- No claim of package correctness, realization success, or reproducibility.
- The toolchain is a binary trust input, not a source-built compiler.
- fix is alpha-quality upstream software; x86_64 Linux is the only supported system here.
