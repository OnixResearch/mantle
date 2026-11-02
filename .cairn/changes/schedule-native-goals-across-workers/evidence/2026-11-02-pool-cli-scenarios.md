# Pool CLI compatibility scenarios run (2026-11-02)

Scope: S12 scenarios executed on the current tree. Acceptance and completion
are not claimed.

## Results

```text
cargo test -p mantle --test smoke pool_plan_validates_selection_without_tickets
  -> ok. 1 passed

cargo test -p mantle --test smoke pool_mode_rejects_unsupported_modes_before_build
  -> ok. 1 passed

cargo test -p mantle --test example_projects production_workflow_projects_evaluate_and_keep_negative_paths
  -> ok. 1 passed (evaluates the signed-cache-roundtrip project)

cargo test -p mantle --test example_projects signed_cache_roundtrip_substitutes_through_an_admitted_trusted_cache
  -> ok. 1 passed (producer build + store push + StaticCacheServer + admitted
     substituter binding; consumer built_total 0 and one recorded substitution)
```

## Coverage

- Signed-cache hit: `signed_cache_roundtrip_substitutes_through_an_admitted_trusted_cache`
  verifies a trusted substituter hit with no local build.
- Client-store cache hit: `production_pool_second_build_hits_the_client_cache`.
- Single-builder and local/pool parity: their own tests plus the retained parity
  digest report.

## Signed-cache workflow attempt

Running the documented `examples/projects/signed-cache-roundtrip` workflow
(producer build with `fixtures/cache.key`, `store push --all`, busybox httpd,
consumer build with `--substituters` and `--trusted-public-keys`) reached the
consumer step but failed before substitution:

```text
{"error":"opening store: store: store-source-admission-required","code":3,"kind":"internal"}
```

The current binary requires an operator source-admission policy for substituters;
the example README does not supply one. The integration test supplies the
admission through `--remote-binding-config` and passes.

## Non-claims

- No S12 completion, acceptance, or archive claim.
- The unsupported-mode test name is `pool_mode_rejects_unsupported_modes_before_build`;
  an earlier mistyped filter matched zero tests and is not evidence.
