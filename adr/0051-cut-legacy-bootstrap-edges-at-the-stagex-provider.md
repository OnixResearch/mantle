# ADR 0051: Cut legacy bootstrap edges at the StageX provider

## Status

Accepted (2026-07-30)

## Context

The protected StageX transition now publishes a checked intermediate provider. The later native GCC graph still names historical derivations for TinyCC, musl, binutils, Make, Bash, and basic POSIX tools. Evaluating those names rebuilt a parallel legacy chain from the old provider inputs. A fresh StageX publication therefore did not prove that later native construction consumed StageX.

Removing every historical derivation name from all later recipes would create a large migration and obscure the existing compiler progression. Importing old output directories would violate the source-built proof authority.

## Decision Drivers

- Make StageX consumption mandatory for the native-provider graph.
- Keep existing later compiler recipes and expected input names stable.
- Do not import, discover, or reuse historical provider outputs.
- Keep each compatibility operation small and reviewable.
- Preserve StageX transition and provider claims without extending them.

## Decision

Mantle treats the fresh StageX transition tree and intermediate provider as the only authorities for the historical compatibility inputs used by later native stages.

The source-built proof binds the original native manifest and the exact StageX source bundle as separate authorities. It reserves logical store paths for both fresh outputs. It executes the protected transition, checks the accepted report identity, adopts that tree into the fresh store, publishes the StageX provider at its accepted logical path, and then builds the later native graph.

Historical compatibility derivations for Stage0 POSIX tools and GNU Make become thin adapters. Each adapter copies a bounded layout from the fresh StageX transition. The adapters keep historical output names so later recipes do not infer a new authority or require a broad rename.

The Bash, TinyCC self-host, TinyCC musl-v2, native musl, and early binutils files remain the exact authenticated recipe sources used inside the protected StageX transition. They are not downstream compatibility inputs. The later native graph cannot evaluate them as provider edges.

`gcc-4.0-native.ncl` consumes the fresh StageX roots directly. This makes the first conventional GCC boundary explicit and prevents it from selecting the legacy TinyCC, musl, binutils, Make, Bash, or utility derivations.

## Alternatives Considered

### Keep both bootstrap graphs and compare outputs

Rejected because output comparison does not prove that the accepted native provider descends from the fresh StageX authority.

### Rewrite every later recipe to new StageX-specific names

Rejected for this change because it creates a large mechanical migration without strengthening the authority boundary beyond the adapter cut points.

### Import a previously published StageX provider

Rejected because the fixed-point requirement requires fresh construction in the current empty output authority.

## Consequences

- Later native builds fail if the fresh StageX roots are absent or have the wrong logical identities.
- Historical downstream derivation names remain compatibility surfaces, not independent provider claims.
- The StageX Bash, TinyCC, musl, and binutils recipes remain authenticated transition sources and are unreachable from the later native graph.
- Adapter outputs add ordinary derivation nodes, but their only bootstrap input is the fresh StageX transition.
- This decision does not prove compiler correctness, StageX completeness outside its checked roles, native-provider admission, the Rust provider, the Mantle fixed point, or release eligibility.
