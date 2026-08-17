## ADDED Requirements

### Requirement: Mantle provides declared Nickel export actions [r[build_correctness.nickel_export_action]]

Mantle MUST provide a bounded Nickel export action that evaluates declared Nickel sources to a requested external format and emits a deterministic evaluation receipt. The receipt MUST bind root source refs, dependency refs, import-path policy, evaluator identity, selected format, output digest, and bounded non-claims.

#### Scenario: Declared export produces receipt-bound output [r[build_correctness.nickel_export_action.scenario.receipt]]

- GIVEN a Nickel export request declares source files, dependency files, import paths, output format, output target policy, and evaluator identity
- WHEN Mantle evaluates the request
- THEN the export receipt MUST bind the declared source closure, import policy, evaluator identity, format, and output digest
- AND downstream action specs MAY cite that receipt as Nickel evaluation provenance.

#### Scenario: Unsafe import path fails closed [r[build_correctness.nickel_export_action.scenario.import-escape]]

- GIVEN an export request includes an absolute import path or an import path that normalizes above the declared root
- WHEN Mantle validates the request
- THEN Mantle MUST reject the request before invoking the Nickel evaluator
- AND it MUST report a deterministic import-path diagnostic without reading escaped files.

#### Scenario: Undeclared dependency blocks strong export claim [r[build_correctness.nickel_export_action.scenario.undeclared-dependency]]

- GIVEN a Nickel export depends on a source file outside the declared source and dependency closure
- WHEN Mantle requires export provenance for a strong correctness claim
- THEN Mantle MUST fail closed with an undeclared-dependency diagnostic
- AND it MUST NOT emit a receipt that implies the output is fully declared.

### Requirement: Mantle records Nickel evaluator toolchain facts [r[build_correctness.nickel_toolchain_provider]]

Mantle MUST model the Nickel evaluator used by export and build-evaluation paths as explicit toolchain data. The descriptor MUST include evaluator binary identity, version, and evaluator options, and MUST be recorded separately from frontend or Bazel-specific toolchain mechanisms.

#### Scenario: Evaluator descriptor participates in identity [r[build_correctness.nickel_toolchain_provider.scenario.identity]]

- GIVEN two Nickel export requests differ only by evaluator binary identity, version, or evaluator options
- WHEN Mantle computes their evaluation receipt identity
- THEN the receipts MUST differ
- AND a receipt from one evaluator descriptor MUST NOT satisfy a request using the other descriptor.

#### Scenario: Toolchain descriptor remains frontend neutral [r[build_correctness.nickel_toolchain_provider.scenario.frontend-neutral]]

- GIVEN an external frontend or adapter supplies a Nickel evaluator descriptor
- WHEN Mantle records the descriptor in export evidence
- THEN Mantle MUST treat binary identity, version, and options as data
- AND it MUST NOT require Bazel repository rules, Nix flakes, Onix module semantics, or other frontend-specific toolchain machinery in core.
