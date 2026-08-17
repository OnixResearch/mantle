## ADDED Requirements

### Requirement: Mantle build-tool boundary

r[build_tool_boundary.mantle_not_module_layer] Mantle MUST remain a frontend-neutral build tool and MUST NOT own Onix/NixOS-style module-layer semantics.

#### Scenario: external frontend hands Mantle build inputs

GIVEN an external frontend such as Onix has evaluated its own module layer and produced concrete derivations or build-plan inputs
WHEN it invokes Mantle
THEN Mantle MUST realize those build inputs through build/store APIs
AND Mantle MUST NOT require machine-role, tag-expansion, provider-policy, upstream-export, or settings-contract semantics to be present in Mantle core.

#### Scenario: raw module-layer input is not a Mantle build input

GIVEN a caller passes raw Onix inventory/module concepts directly to Mantle's build-tool boundary
WHEN Mantle validates the request
THEN Mantle MUST fail or route the request to an explicitly external module-layer tool
AND it MUST NOT silently interpret raw Onix roles, tags, providers, packages, artifacts, or upstream exports.

#### Scenario: implementation surface is guarded against module-layer coupling

GIVEN Mantle implementation or public handoff files change
WHEN boundary tests scan the CLI, stdlib, workspace metadata, implementation sources, docs, and examples
THEN they MUST reject reintroduced in-tree module-layer surfaces such as `mantle system`, `crunch-system`, `SystemModule`, Onix module repos, or NixOS module evaluators
AND they MUST keep accepted frontend handoff examples build-shaped.

### Requirement: Onix-owned module lowering

r[build_tool_boundary.onix_owns_module_lowering] Onix or an Onix-owned adapter MUST own module ABI, module implementation invocation, settings validation, upstream/provider topology, package/artifact selection, and lowering into Mantle build inputs.

#### Scenario: Onix evaluates modules before Mantle build

GIVEN an Onix service module with role settings, defaults, contracts, upstream dependencies, and provider exports
WHEN Onix uses Mantle as its backend
THEN Onix MUST evaluate those module semantics before calling Mantle
AND Mantle MUST receive only frontend-neutral build inputs or opaque data produced by that evaluation.

#### Scenario: Onix diagnostics remain in the module layer

GIVEN invalid Onix settings or missing Onix upstream/provider data
WHEN the Onix module layer evaluates the configuration
THEN diagnostics MUST be produced by the Onix-owned module layer
AND Mantle MUST NOT need Onix-specific diagnostic rules to report those module-layer failures.

### Requirement: Synthetic system scaffold is not an integration contract

r[build_tool_boundary.synthetic_system_eval_not_integration] Mantle MUST NOT treat the current synthetic `system eval` scaffold as the production Onix module integration path.

#### Scenario: fake fragments are not deployable evidence

GIVEN Mantle's current `system eval` path returns fragments that were not produced by real module implementations
WHEN deployable artifact work is planned
THEN those fragments MUST NOT be used as evidence that Onix modules run on Mantle
AND the work MUST depend on an Onix-owned module layer that lowers into Mantle build inputs.

#### Scenario: system scaffold is quarantined

GIVEN the `system eval` scaffold remains in the Mantle tree
WHEN operators or tests describe supported Onix integration
THEN the scaffold MUST be documented or guarded as experimental/demo-only
AND it MUST NOT expose a stable Onix/NixOS-style module ABI from Mantle core.
