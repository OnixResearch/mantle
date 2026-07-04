## ADDED Requirements

### Requirement: Receipt-bound executable search path

r[build_correctness.receipt_bound_path] Mantle MUST derive strict build executable search paths from declared tool references or accepted host-tool inventory records, and receipts MUST bind the ordered search path, alias views, and real tool identities used for execution.

#### Scenario: declared tool path is accepted

GIVEN a strict build action declares tool references needed by the builder
WHEN Mantle constructs the build search path
THEN every PATH entry MUST map to a declared tool ref or accepted host-tool inventory record
AND the build receipt MUST bind the ordered path digest and real tool identities.

#### Scenario: ambient path poisoning fails closed

GIVEN the parent PATH contains an undeclared directory or executable that would shadow a declared tool
WHEN Mantle prepares a strict build
THEN Mantle MUST reject the unclassified PATH influence before execution
AND it MUST NOT search the ambient PATH to repair missing declared tools.

#### Scenario: alias identity remains explicit

GIVEN Mantle materializes stable executable aliases inside a sandbox
WHEN a build or proof report records executable authority
THEN the report MUST distinguish alias paths from the real content-addressed tool refs or host-tool inventory records
AND alias names MUST NOT satisfy tool identity without matching real refs.
