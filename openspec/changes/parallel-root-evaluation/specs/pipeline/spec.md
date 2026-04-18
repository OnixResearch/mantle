## MODIFIED Requirements

### Requirement: Pipeline owns eval and convert

The pipeline MUST evaluate Nickel in-process through `crunch-eval`, but it
MUST NOT require a whole-program export-ready deep evaluation merely to obtain
root labels.

The build path MUST:
- discover root labels through the lazy `crunch-eval` boundary first,
- force derivation values per root only when needed for conversion,
- allow bounded overlap between multi-root forcing/conversion and downstream
  worker scheduling when more than one root is requested, and
- continue to use direct typed derivation extraction rather than a whole-program
  JSON export string for build execution.

`evaluate_to_json()` remains available for `crunch eval` and other debug or
reporting paths, but the build path MUST use the lazy discovery + selected-root
forcing representation that `crunch-eval` exposes.

The pipeline MUST still run `crunch_glue::convert()` for each derivation and
bridge the `ConversionCache` to a `DerivationRegistry`.

The pipeline MUST serialize `crunch_glue::convert()` calls against the shared
`ConversionCache` and `DerivationRegistry`, and it MUST pass the configured
store prefix to all conversion call sites.

The first iteration of pipeline-side eval parallelism MUST be bounded by an
internal cap no greater than the configured build parallelism. When the build
configuration does not set `max_jobs`, the pipeline MUST derive its build and
eval caps from the existing default build-parallelism resolution and still keep
eval parallelism at least `1`.

For streaming execution, the pipeline MUST treat each root-force request as an
independent error-handling unit. It MUST NOT depend on partial success results
from a failed multi-root request.

#### Scenario: Multi-root build streams converted roots incrementally

- GIVEN a `.ncl` file exporting multiple root derivations
- WHEN `build()` is called
- THEN the pipeline discovers root labels before per-root forcing begins
- AND it sends converted roots to the worker incrementally as they are ready
- AND it does not wait for a full serial evaluated-root vector before worker
  scheduling can start
- AND it preserves label -> derivation association even if converted roots are
  streamed in completion order rather than request order

#### Scenario: Eval parallelism stays bounded by build parallelism

- GIVEN `build()` runs with a configured `max_jobs`
- WHEN the pipeline chooses its eval parallelism
- THEN the eval parallelism does not exceed that configured build parallelism
- AND the pipeline does not create unbounded root-force tasks

#### Scenario: Root forcing failure stops later dispatch without losing prior labels

- GIVEN the pipeline has already streamed one converted root to the worker
- AND a later independent root-force request fails during bounded evaluation
- WHEN `build()` surfaces that evaluation failure
- THEN the pipeline stops dispatching additional not-yet-converted roots
- AND it preserves label association for any root already sent downstream
- AND the final build result reports the labeled evaluation failure
