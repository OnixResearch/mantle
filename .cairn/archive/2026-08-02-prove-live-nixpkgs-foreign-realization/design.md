# Design: cache-only live Nixpkgs realization

## Decision

Use a separate cache-only executable-plan route. Preserve exact foreign output
paths only when the target prefix equals the sole declared source prefix.
Hydrate the selected root and its signed runtime-reference closure before the
normal scheduler runs.

The route never executes a foreign builder. A cache miss is a terminal route
failure, not permission to rebuild.

## Functional core

Pure functions own these decisions:

- policy mode and unchanged-prefix admission;
- exact preserved output and source path selection;
- executable-plan route validation;
- deterministic closure work ordering and limit decisions;
- cache hydration fact ordering;
- realization receipt classification.

The core accepts values and returns values or typed errors. It does not read
files, use the network, mutate the store, or inspect the environment.

## Imperative shell

The realization shell opens the configured Mantle store. It asks the existing
Nix HTTP PathInfo service for each bounded closure member. That service verifies
the configured cache signature and NAR before returning PathInfo and castore
content.

The shell exports closure content under the selected physical store directory.
It then invokes the normal derivation registry, scheduler, worker, and store.
All selected outputs must already be admitted, so no builder can run.

## Identity and receipt

The executable plan records `cache-only-preserve-v1`. Exact output maps are
identity maps. The plan identity binds this route and the execution profile.

The realization receipt records:

- cache-only route identity;
- ordered cache URLs and trust configuration;
- every hydrated or reused closure PathInfo;
- NAR hash, size, references, signature names, and transfer disposition;
- units not required by the selected runtime closure;
- scheduler build-report identity;
- explicit non-claims.

## Rejected alternatives

### Rebind Nix cache content to `/mantle/store`

This breaks embedded store references in ELF files, scripts, links, and data.
Generic byte rewriting cannot preserve arbitrary NAR semantics.

### Producer-side Mantle cache conversion

This adds a second cache format and local signing authority before the direct
Nix cache boundary has been proved. It also has the same embedded-reference
problem unless paths stay `/nix/store`.

### Local rebuild of the complete derivation graph

The live graph contains a large build dependency closure. This route would test
bootstrap compatibility, not the bounded substitution claim. It remains future
work.

## Failure rules

- Preserve mode with a changed or multiple source prefix fails plan admission.
- Cache-only realization requires substitution and rejects offline mode.
- The source bundle must be empty because no builder input can be consumed.
- Missing or invalid cache content stops before scheduler execution.
- Closure limits stop hydration and produce bounded failure evidence.
- The route must not fall back to local build or remote execution.
