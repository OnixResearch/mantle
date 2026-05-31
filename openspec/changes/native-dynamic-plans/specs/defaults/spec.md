## MODIFIED Requirements

### Requirement: Dynamic derivations as default
Mantle MUST support dynamic derivations as native dynamic build-plan artifacts, meaning builds can produce declared `mantle-plan-v1` outputs whose validated units are registered and scheduled after the producer build completes.
ID: defaults.dynamic.derivations


The native dynamic feature MUST NOT require Nix `.drv` ATerm parsing, Steel, or evaluator suspension. Compatibility support for `.drv` outputs MAY remain available, but it MUST be reported separately from native dynamic plans and MUST NOT define the native ABI.

The build orchestration MUST:

- Inspect only declared dynamic-plan outputs after producer completion
- Decode plan artifacts into Mantle-owned typed data
- Validate bounded schema, dependencies, output declarations, environment, and store-prefix policy before registration
- Register accepted units as build-engine goals through the lazy worker
- Ignore undeclared plan-looking outputs without scheduling them or recording native dynamic-plan rows
- Reject malformed, missing, non-regular, over-limit, or policy-violating declared plan outputs without treating them as ordinary successful dynamic roots
- Record the producer, plan artifact path when present, plan digest, accepted unit IDs, rejected reason, and compatibility-vs-native mode in build diagnostics

#### Scenario: Build produces a native plan

- GIVEN a derivation declares a dynamic-plan output named `plan`
- AND the completed build writes a valid `mantle-plan-v1` artifact to that output
- WHEN the build engine finishes the producer
- THEN Mantle validates the artifact
- AND registers the artifact units as native goals
- AND schedules those goals without re-entering Nickel evaluation

#### Scenario: Undeclared plan is ignored

- GIVEN a derivation does not declare any dynamic-plan outputs
- WHEN its build writes bytes that decode as `mantle-plan-v1`
- THEN Mantle does not auto-schedule those bytes as a dynamic plan
- AND the build report records no native dynamic plan row for that output

#### Scenario: Compatibility drv output stays separate

- GIVEN a derivation produces a Nix `.drv` ATerm output
- WHEN compatibility detection is enabled
- THEN Mantle MAY parse and schedule it through the compatibility path
- AND the report labels it as compatibility dynamic discovery, not native `mantle-plan-v1`
