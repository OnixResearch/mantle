# Baseline

Date: 2026-08-03

## Revision

- `origin/main`: `332b62a98bc8b6239461364cab171897f45285e0`
- lifecycle scaffold commit: `a9e6f78a9be930331c78a024b5bd0831550ce20d`

## Focused tests before implementation

The existing `crunch-release-core` tests passed before the core change.

The existing release-evidence shell tests passed before the shell change.

The host did not expose `cargo` on `PATH`. Validation then used Mantle's available nightly toolchain with the required clang and mold paths.

## Nix baseline blocker

Command:

```text
nix develop path:$PWD -c true
```

Result: blocked before entering the shell.

```text
error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'
```

This dependency existed before the chapter transport change. Focused Cargo checks use already available locked dependencies and local toolchain paths.
