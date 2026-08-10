# Nix derivation compatibility boundary

Mantle parses concrete Nix `.drv` files through one reviewed adapter.
The adapter is `src/nix_derivation_adapter.rs`.

This boundary applies only to standard `/nix/store` derivations.
Mantle still uses its adapted `nix-compat` code for native derivations, configurable prefixes, store protocols, NARInfo, castore, and daemon integration.

## Admitted dependency

The admitted package facts are:

- crate: `nix-derivation`;
- version: `0.1.0`;
- upstream commit: `2cfc0f90ed83ea3cc983e5c305f89494a6df073e`;
- crates.io SHA-256: `a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074`;
- license: `Apache-2.0`;
- recorded Nix compatibility: Nix 2.34.

`config/nix-derivation-admission.ncl` records these facts and the rollback state.

Run the boundary self-test:

```text
nix develop -c env CARGO_TARGET_DIR=target/cargo-script \
  cargo -Zscript scripts/check-nix-derivation-boundary.rs --self-test
```

Run package-source parity after you obtain the exact crate package and upstream commit:

```text
nix develop -c env CARGO_TARGET_DIR=target/cargo-script \
  cargo -Zscript scripts/check-nix-derivation-boundary.rs \
  --root . \
  --package-root "$PACKAGE_ROOT" \
  --upstream-root "$UPSTREAM_ROOT" \
  --crate-archive "$CRATE_ARCHIVE"
```

The guard compares the packaged source projection with the reviewed upstream tree. It also rejects production imports outside the adapter.

## Admission flow

The shell reads each `.drv` file under fixed byte and file-count limits.
It then passes owned bytes and the logical store identity to the pure adapter.

The adapter performs these steps:

1. Validate the `/nix/store/*.drv` identity.
2. Derive the out-of-band derivation name.
3. Parse the complete ATerm input.
4. Check collection, field, edge, and dynamic-depth limits.
5. Classify every output and dynamic-input form.
6. Convert accepted data to Mantle-owned values.
7. Reject values that the current foreign graph cannot represent.

The shell publishes artifacts only after the complete closure passes.
Backend-produced closures use the same adapter after process and output limits pass.

## Limits

The initial limits are:

- 16 MiB for one derivation;
- 256 MiB for one derivation bundle;
- 2,048 derivations in one bundle;
- 256 items in one derivation collection;
- 256 input edges;
- 256 dynamic-input levels;
- 32 KiB for an ordinary field;
- 1 MiB for structured attributes.

The adapter rejects empty input, arithmetic overflow, and every exceeded limit.
The shell bounds file reads before parsing.

## Supported projection

The current foreign graph accepts traditional input-addressed outputs and flat or NAR fixed outputs with concrete paths.
It preserves structured `__json` bytes and ordered fetch-candidate data.

The current projection rejects these forms:

- floating outputs;
- deferred outputs;
- impure outputs;
- text-addressed fixed outputs;
- Git-addressed fixed outputs;
- recursive dynamic-input graphs;
- non-UTF-8 environment values at the string-based foreign graph boundary;
- logical-path and environment-name mismatches.

These forms remain parser facts. Mantle does not flatten or silently discard them.
Guix and other prefix-rewrite paths remain on the prior parser.

## Hash domains

Nix derivation, output, placeholder, and store-path operations use Nix-required algorithms and the standard prefix.
Mantle does not replace these SHA-256 facts with BLAKE3.

Mantle graph, policy, plan, receipt, and evidence identities continue to use labeled BLAKE3 roles.
An identical byte sequence cannot change its digest role implicitly.

## Rollback

Use revision `465e4b45abd7ff8887ada01df93c9a45d30faf06` as the pre-adoption state.
Remove the exact dependency and restore the prior parser call sites together.
Do not remove only the dependency or only the adapter.

Foreign graph, package-index, receipt, and CLI schemas do not change during rollback.

## Non-claims

Passing this boundary proves agreement only for the recorded package, Nix version, fixtures, limits, and accepted projection.
It does not prove arbitrary Nix compatibility, evaluator parity, build success, builder safety, output correctness, store trust, reproducibility, or release eligibility.
