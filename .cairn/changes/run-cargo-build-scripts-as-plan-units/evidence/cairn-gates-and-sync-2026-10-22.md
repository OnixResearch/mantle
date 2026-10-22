
# Cairn validation, gates, and accepted spec synchronization (2026-10-22).

$ cairn validate --root .
valid: True issues: []

$ cairn gate proposal run-cargo-build-scripts-as-plan-units --root .
valid: True issues: []

$ cairn gate design run-cargo-build-scripts-as-plan-units --root .
valid: True issues: []

$ cairn gate tasks run-cargo-build-scripts-as-plan-units --root .
valid: True issues: []

$ cairn sync run-cargo-build-scripts-as-plan-units --root .
valid: None issues: None

$ cairn sync run-cargo-build-scripts-as-plan-units --root . --execute
valid: None issues: None


# Accepted spec synchronization is blocked by an unarchived dependency.

$ cairn sync run-cargo-build-scripts-as-plan-units --root .
  blocked: true
  reasons: ["run-cargo-build-scripts-as-plan-units: change_dependency.unsatisfied_dependency: dependency build-cargo-units-as-dynamic-plans is not archived"]

The validation and the proposal, design, and task gates all report valid with no
issues. The synchronization and the pre-archive readiness therefore remain
blocked until the build-cargo-units-as-dynamic-plans change is archived.
