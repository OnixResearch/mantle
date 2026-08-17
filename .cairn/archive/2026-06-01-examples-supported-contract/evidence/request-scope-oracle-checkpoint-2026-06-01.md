# Request scope oracle checkpoint

Question: What did the user's ambiguous request `write cairns for this` refer to, and are the four examples-focused Cairn changes the correct decomposition?

Inspected evidence:

- Immediately preceding user request: `how can we harden and expand our examples`.
- Immediately preceding assistant response recommended treating `examples/` as supported API and named seven work areas: catalog/source of truth, tests for every example, deterministic fetch examples, negative examples, built-output execution, doc drift rail, and expanded beginner/fetcher/composition/project/bootstrap/trust/advanced lanes.
- Current checked-in examples and tests inspected before scaffolding: `examples/README.md`, root `README.md` examples section, `tests/examples_eval.rs`, `tests/examples_build.rs`, and `tests/integration_build.rs` example coverage.
- Created changes:
  - `examples-supported-contract` covers catalog/source of truth, README drift, support tiers, and Mantle naming.
  - `examples-deterministic-validation` covers eval/build/output execution matrix plus positive/negative validation rails.
  - `examples-fetcher-fixture-hardening` covers deterministic offline fetcher fixtures, fixed-output mismatch, and repair workflow coverage.
  - `examples-workflow-gallery` covers progressive beginner-to-advanced gallery, package/project lanes, and trust/provenance examples.

Decision: The target of `this` is the examples hardening/expansion plan from the immediately preceding turn. Four Cairn changes are the smallest useful decomposition because the work separates into independent catalog/docs contract, validation rail, fetcher fixture hardening, and gallery expansion/provenance lanes. No task is complete; the changes are scaffolds only.

Owner: Mantle maintainer / next implementation agent.

Next action: Work the changes independently in dependency order: start with `examples-supported-contract`, then use the catalog to drive `examples-deterministic-validation`, `examples-fetcher-fixture-hardening`, and `examples-workflow-gallery`. Keep task boxes unchecked until durable implementation evidence exists.
