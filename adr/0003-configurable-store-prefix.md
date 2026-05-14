# ADR 0003: Configurable Store Prefix

## Status

Accepted (2026-04-06)

Rename amendment (2026-05-14): this ADR records the pre-rename Crunch decision.
The current Mantle default store prefix is `/mantle/store`; explicit
`/crunch/store` remains a legacy compatibility prefix, and `--nix-compat` still
selects `/nix/store` for interop testing.

## Context

crunch hardcoded `/nix/store` as the store path prefix everywhere — in
derivation hash computation, ATerm serialization, path formatting, and
sandbox environment variables. crunch already produces incompatible
ATerm hashes vs Nix (BLAKE3, different `outputs` env var), so there's
no real compatibility benefit from sharing the prefix.

The hardcoded prefix also meant crunch depended on two Nix-provided
binaries (bwrap, busybox-static) from `/nix/store` paths. Removing
these dependencies requires the prefix to be configurable.

## Decision

The store prefix is a runtime parameter (`--store-prefix`, default
`/crunch/store`). It flows through the entire pipeline:

```
CLI --store-prefix
  -> ConversionCache(store_dir)
  -> DerivationRegistry(store_dir)
  -> StoreConfig { store_dir }
  -> StoreHandle { store_dir }
  -> Builder { store.store_dir() }
  -> Worker { known_paths.store_dir() }
  -> BuildRequest { store_dir }
  -> sandbox NIX_STORE env var
```

The vendored nix_compat crate gained `_with_store_dir()` variants for
all hash-affecting functions (ATerm serialization, derivation path,
output paths, FOD digest, HDM). The original functions still exist
and default to `/nix/store` for API compatibility.

`--nix-compat` is a shorthand for `--store-prefix=/nix/store` for
interop testing.

## Consequences

- All existing pathinfo.redb databases are invalidated (paths have
  different prefixes). Users must clear their state directory.
- crunch outputs are no longer confusable with Nix outputs. Paths
  starting with `/crunch/store/` are unambiguously crunch's.
- Binary cache substitution from cache.nixos.org was already broken
  (different ATerm hashes). The prefix change doesn't make this worse.
- The self-build pipeline now bootstraps its own busybox and bwrap,
  eliminating the last Nix binary dependencies.

## Alternatives Considered

**Compile-time constant**: Simpler but requires recompiling to change
the prefix. Can't test with multiple prefixes in the same binary.

**Thread-local / global state**: Hides the dependency. Testing with
multiple prefixes in parallel becomes racy.

**Keep `/nix/store`**: Perpetuates confusion about Nix compatibility
that doesn't exist.
