# Foreign Derivation Import Delta

## ADDED Requirements

### Requirement: Producer graph admission follows the producer tool-graph contract

r[foreign_derivation_import.producer_graph_bounds] Foreign-import admission MUST
bound producer graphs by the producer tool-graph contract rather than by
fixture-sized limits. The derivation bound MUST equal the verification bridge
producer bound (`crunch_verification_bridge_core::producer::MAX_TOOL_DERIVATIONS`,
32,768), the source-payload bound MUST admit the producer export contract's
16,384 source objects, the hash-domain record bound MUST be derived from the
node bound (one Nix-identity record per node), the `prepare-sources` read budget
MUST cover the source-payload bound plus the command's fixed reads, the per-node
input-edge and foreign-import field-byte bounds MUST admit the measured producer
surface, and the scheduler goal and derivation-registry bounds MUST admit every
node of an admitted plan (one goal and one registration per planned unit).

#### Scenario: A real nixpkgs closure is admitted

GIVEN a producer export declares a reachable closure of 32,768 derivations
AND at most 16,384 source payloads
WHEN `produce-nix` lowering, graph validation, and executable planning run
THEN they MUST accept the graph
AND the emitted hash-domain records MUST fit the derived record bound
AND compilation MUST complete without quadratic per-node rebuilds of the exact
path maps.

#### Scenario: One node past a bound fails closed

GIVEN an export declares 32,769 derivations, or a graph declares 16,385 source
payloads
WHEN admission runs
THEN it MUST reject the input with `nix-derivation-count-out-of-range` or
`source-payload-limit-exceeded` respectively
AND it MUST NOT truncate the closure or approximate the surplus records.

#### Scenario: An admitted plan is schedulable end to end

GIVEN an admitted plan with up to 32,768 native units
WHEN foreign realization registers derivations and creates scheduling goals
THEN the goal and registry bounds MUST admit every planned unit
AND neither MUST reject a plan that admission accepted.

#### Scenario: The bound tie is test-visible

GIVEN either the bridge producer bound or the foreign-import node bound changes
WHEN the `mantle` unit tests run
THEN a test comparing the two constants MUST fail
AND the mismatch MUST NOT be resolved by editing generated artifacts.
