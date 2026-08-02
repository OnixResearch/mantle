# Recorded validation

Validation ran on 2026-08-02 with the pinned inputs in `workflow.ncl`.

This record is local integration evidence. It is not release evidence.

## Cache import

The cache-only realization completed with these facts:

- Receipt status: `complete`
- Realization route: `cache-only-preserve-v1`
- Receipt BLAKE3: `2aacaabcf1eade059968fd6a8997ae71e0f432ab646c2c09d49fef3c32f215f0`
- Plan BLAKE3: `99f2d282f31771f47287d04e1071eb1ffd9e8aca7f1b547e7587c6d58b1c6a9a`
- Import receipt BLAKE3: `694503d7248323657d38a79731340c8c474f739206fe456b9479aaf82f3ef54e`
- Source bundle BLAKE3: `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777`
- Runtime closure members: `5`
- Closure disposition: `remote-substituted` for all members

The imported root was `/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c`.

## Builder execution

The final consumer run used the rebuilt Mantle binary. Before execution, these checks passed:

- The exact `mksh` path was absent from the host `/nix/store`.
- `nix` was absent from the consumer command `PATH`.
- `nix-store` was absent from the consumer command `PATH`.

The strict build report contained these facts:

- `built_total`: `1`
- `cached_total`: `0`
- `failed_total`: `0`
- `hermeticity_mode`: `strict`
- `hermeticity_audit_events`: empty
- Recorded search-path entries: empty

The consumer output was:

```text
builder=/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c/bin/mksh
provenance=cache-imported
workflow=nixpkgs-tool-use
execution=imported-builder
```

These facts show that the imported executable ran. A cache hit did not produce the consumer output.

## Negative evidence

A fresh state and fresh physical output directory rejected the consumer. The report used `missing-source-input` and named the exact imported `mksh` path.

The focused `crunch-store` tests also passed:

- `http_closure_untrusted_root_fails_before_nar_download`
- `http_closure_changed_dependency_nar_keeps_closure_absent`

The first test rejects an untrusted NARInfo signer before NAR transfer. The second test changes NAR bytes and keeps all affected PathInfo absent.

## Supporting checks

These checks passed:

- Full `crunch-build` test suite
- `crunch-build` Clippy with `--all-targets --no-deps -- -D warnings`
- Example inventory: `17` tests
- Example workflow gallery: `12` tests
- Rust formatting check
- `git diff --check`

Broad Clippy stopped on existing lints in vendored code and `crunch-store/src/provenance.rs` tests. The changed package code and focused tests passed.

`nix flake check -L --keep-going` did not pass. Unchanged release checks excluded two tracked content-bound fixture files from their Nix build source. The `artifact-auth-radicle-cutover` check also failed without a build log.

## Claim boundary

This evidence proves one bounded cache import and one Mantle consumer build. It does not make `mksh` Mantle-built.

It does not prove Nix evaluator parity, package correctness, reproducibility, runtime safety, deployment, or release eligibility.
