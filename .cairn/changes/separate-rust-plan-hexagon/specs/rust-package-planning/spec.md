# Rust Package Planning Hexagonal Architecture Delta

## ADDED Requirements

### Requirement: Rust package planning has a strict functional core

r[rust_package_planning.hexagonal_core] Mantle MUST keep package admission, feature resolution, dependency selection, host and target classification, unit topology, action planning, compatibility classification, deterministic identities, blockers, and receipt preimages in a `no_std + alloc` core over normalized bounded facts.

#### Scenario: Equivalent workspace facts produce one plan

GIVEN equivalent admitted package, manifest, lock, target, feature, toolchain, and artifact facts in different collection orders
WHEN the Rust-planning core constructs a plan
THEN it MUST return the same units, edges, actions, blockers, identities, and receipt preimages
AND it MUST not read files, paths, processes, environment state, Cargo state, rustc state, caches, stores, clocks, or presentation settings.

#### Scenario: Host authority enters the planning core

GIVEN the Rust-planning core imports filesystem, process, environment, Cargo, rustc, store, cache, path, async-runtime, CLI, or rendering authority
WHEN architecture checks run
THEN they MUST fail with the authority class and dependency path
AND a convenience adapter inside the core crate MUST NOT bypass the failure.

### Requirement: Rust-plan ports are application-owned

r[rust_package_planning.application_owned_ports] Rust-plan application ports MUST use Mantle-owned requests, observations, results, and capability errors for workspace facts, Cargo oracle capture, compiler inspection, unit execution, and cache access. They MUST NOT use `RunError`, provider SDK types, raw store services, or process types.

#### Scenario: Cargo oracle material reaches the planner

GIVEN the Cargo process adapter captures bounded metadata and unit-graph bytes
WHEN the Rust-plan application submits oracle facts to the core
THEN the adapter MUST decode them into bounded structural facts and the core MUST admit their semantic values
AND Cargo process or JSON implementation types MUST NOT enter the core contract.

#### Scenario: A CLI error enters a port

GIVEN a Rust-plan port returns `RunError` or another presentation-owned error
WHEN API-shape checks run
THEN they MUST reject that dependency direction
AND the capability must return a typed application or adapter error instead.

### Requirement: Rust unit effects are explicit

r[rust_package_planning.explicit_unit_effects] The Rust-planning core MUST return ordered bounded unit effects with declared arguments, environment facts, input identities, expected outputs, and limits. The shell MUST execute Cargo, rustc, filesystem, and cache effects through explicit adapters and MUST return typed observations for result classification.

#### Scenario: Supported unit is ready to execute

GIVEN a supported admitted unit and all required dependency, toolchain, environment, and output facts
WHEN the core plans unit execution
THEN it MUST return a deterministic bounded unit effect
AND it MUST NOT claim that rustc executed or produced an artifact.

#### Scenario: Adapter reports a failed unit

GIVEN the shell attempted the declared unit effect and rustc returned a bounded failure observation
WHEN the core classifies the observation
THEN it MUST return the stable unit failure and receipt facts
AND it MUST NOT search ambient Cargo or target state as fallback.

### Requirement: Rust-plan extraction preserves accepted compatibility

r[rust_package_planning.hexagonal_compatibility] Hexagonal extraction MUST preserve accepted package selection, feature activation, unit identities, topology, rustc arguments, environment, blockers, cache decisions, receipts, JSON, diagnostics, and CLI behavior unless a separate versioned change authorizes a difference.

#### Scenario: Accepted Cargo parity fixture crosses the new boundary

GIVEN an accepted workspace and Cargo oracle fixture
WHEN legacy and extracted planning paths process equivalent normalized facts
THEN their package, feature, target, unit, action, blocker, identity, and receipt outputs MUST remain equal
AND Cargo-free claims MUST retain their current bounded scope.

#### Scenario: Unsupported behavior remains unsupported

GIVEN a workspace uses unsupported Cargo behavior or lacks required declared material
WHEN the extracted planner evaluates it
THEN it MUST return the accepted deterministic unsupported-boundary blocker
AND it MUST NOT invoke Cargo as hidden build orchestration or broaden support claims.
