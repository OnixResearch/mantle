# Standalone Nickel export core

Mantle uses `nickel-export-core` from
`https://github.com/OnixResearch/nickel-export` at the exact Git revision
`257fafc1c746f1faf156207043a4c826bfb16d49`.

The pin is repeated deliberately across these release-relevant surfaces:

- `Cargo.toml`, `Cargo.lock`, and the exact self-build replacement in
  `.cargo/vendor-config.toml`;
- `flake.nix` and `flake.lock`;
- `config/nickel-export-core-source.ncl` and its generated JSON export.

No branch, tag, version-only dependency, workspace-relative path, Nix path input,
or alternate release override is accepted. Check the complete binding with:

```bash
nix develop -c cargo -Zscript scripts/check-nickel-export-core-pin.rs --root .
nix develop -c cargo -Zscript scripts/check-nickel-export-core-pin.rs --self-test
nix build .#checks.x86_64-linux.nickel-export-core-pin --no-link -L
```

## Authority boundary

The pure adapter in `src/nickel_export_core_adapter.rs` receives only explicit
UTF-8 request fields, exact source/dependency/output bytes, evaluator descriptor,
and diagnostics. It delegates request normalization, admission, BLAKE3 artifact
identity, canonical manifest identity/freshness, and the one-way
`mantle-nickel-export-receipt-v1` projection.

The shell in `src/nickel_export.rs` retains:

- `crunch-eval` invocation and evaluator diagnostics;
- root, path, no-follow file, symlink, and source-byte admission;
- destination creation and writes;
- stdout/stderr and exit behavior;
- all build evidence and release-policy authority.

The adapter never reads files, resolves imports, executes Nickel, inspects the
environment, writes outputs, or makes build/release decisions.

For `mantle export`, normalized source capture, Nickel evaluation, and source
recheck execute only after a bounded application effect plan is accepted. A
file destination has a separate publication plan decided from the evaluator
output digest before writing. The shell reads the published regular file back
through the no-follow port, measures its actual bytes (at most the bounded
probe), and compares its independently computed BLAKE3 identity with the
planned digest before reporting success. A failed write or read-back retains
its capability error and exit code 3; a changed or oversized read-back fails
closed rather than announcing a receipt. Earlier file writes can remain after
failure, so the reported receipt is not a transaction or durable-write proof.
Stdout rendering remains presentation, not a file read-back claim.

## Dual-run and rollback

Legacy and canonical paths receive the same captured source/dependency bytes,
output bytes, evaluator descriptor, and diagnostics. Mantle compares:

- the standalone core's one-receipt canonical manifest identity;
- exact source, dependency, and output identities;
- the full `mantle-nickel-export-receipt-v1` projection.

Drift classes are `request-normalization`, `dependency-closure`,
`evaluator-descriptor`, `serialization`, `mantle-policy`, and `projection`.
Unexplained drift blocks canonical authority. The bounded legacy projection
adapter remains available for rollback, but rollback never permits a path or
floating source override, mixed evaluator evidence, stale/tampered evidence, a
receipt on evaluator failure, secret-marker admission, or weakened non-claims.

Required regression coverage includes a positive exact-source receipt and the
negative path escape, symlink, stale output, source/output tamper, mixed
evaluator, evaluator error, secret marker, and overclaim cases.

## Claim boundary

Passing the cutover proves compatibility for the checked request, exact artifact
identities, canonical one-receipt manifest identity, Mantle v1 projection, and
freshness checks at the pinned revision. It does not prove evaluator equivalence,
a complete evaluator-observed import closure, semantic correctness, build
correctness, deployability, or release eligibility.
