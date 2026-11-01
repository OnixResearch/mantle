# Pool executor factory seam (2026-10-31)

Scope: let the pipeline attach an explicitly selected pool to the ordinary lazy
builder without owning endpoint configuration, credentials, transport, or
executor implementation.

## Change

- `crunch_pipeline::PoolExecutorFactory` takes the pipeline's opened
  `StoreHandle` and returns an `Arc<crunch_build::distributed::PoolDispatch>`.
  The shell implements it; the pipeline only hands over the store capability.
- `BuildConfig.pool_executor_factory: Option<Arc<dyn PoolExecutorFactory>>` is
  absent by default, so every existing caller is unchanged.
- Both Linux builder assembly points (strict cache-only and ordinary sandbox)
  call the factory before the store is split into capabilities and attach the
  result with `Builder::with_pool_dispatch`.
- All `BuildConfig` literals in the repository set the new field.

## Verification

| Gate | Result |
| --- | --- |
| `cargo check -p crunch-pipeline --all-targets`, `cargo check -p mantle --all-targets` | exit 0 |
| strict Clippy `crunch-pipeline`, `cargo fmt --check -p crunch-pipeline` | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

## Not claimed

- No factory implementation exists yet, so `build --builder-pool` still cannot
  run; the bin-side executor that builds the `nix-derivation` request and runs
  the stdio exchange is the next step.
- Compile and lint checks do not exercise routing; the pipeline-level routing
  fixture arrives with the first real factory.
