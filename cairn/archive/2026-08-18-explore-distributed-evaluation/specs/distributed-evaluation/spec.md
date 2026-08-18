## ADDED Requirements

### Requirement: Distributed evaluation feasibility assessment stays evidence-gated

r[distributed_evaluation.feasibility_assessment] Mantle MUST classify a distributed-evaluation feasibility assessment from deterministic evidence. The assessment MUST inventory evaluation seams, analyze candidate routes, bind evidence identities, select one bounded outcome per route, change no evaluation or remote-build authority, and preserve its decision in lifecycle evidence.

#### Scenario: evaluation seam inventory is complete

GIVEN an exploration of distributed evaluation covers the current client-evaluates boundary
WHEN the inventory task records evaluation seams
THEN the inventory MUST name the embedded evaluator, isolated worker sessions, strict worker protocol, evaluation-stream workers, evaluation budget core, portable client boundary, source staging, and remote-build data plane
AND it MUST bind each seam to its source path, accepted spec, or ADR identity.

#### Scenario: candidate routes cover both distribution shapes

GIVEN a feasibility assessment selects candidate routes
WHEN evidence collection runs
THEN the assessment MUST analyze the eval-service route and the evaluate-once producer route
AND it MUST record for each route the source staging, transport, authority, streaming, dynamic-goal, and suspension facts.

#### Scenario: worker boundary probe decides transportability

GIVEN a bounded round-trip probe sends an isolated eval-worker request over one transport
WHEN the probe verifies the returned response against the in-process path
THEN a match within declared bounds MUST admit the boundary as transportable
AND hidden host-local reads, undeclared imports, ambient stdlib dependence, malformed framing, oversized messages, or response disagreement MUST classify the eval-service route as `blocked` or `rejected`.

#### Scenario: decisions select one bounded outcome per route

GIVEN complete inventory, route, and probe facts
WHEN the pure classification core evaluates the facts
THEN it MUST emit for each route exactly one outcome from `candidate`, `rejected`, or `blocked` with deterministic reasons
AND proven transportability with complete evidence MUST produce `candidate`
AND missing, inconclusive, or hidden host-local facts MUST produce `blocked`
AND a demonstrated authority, streaming, or source-identity blocker MUST produce `rejected`.

#### Scenario: exploration changes no authority

GIVEN the feasibility assessment completes
WHEN Mantle records the report, ADR, and oracle checkpoint
THEN evaluation, remote-build dispatch, scheduler, provider, and publication behavior MUST remain unchanged
AND a `candidate` outcome MUST authorize only a later Cairn change with separate implementation and admission evidence.

#### Scenario: assessment claims remain bounded

GIVEN operators or maintainers cite the assessment
WHEN they describe its result
THEN they MUST identify the inventory, candidate routes, probe, evidence, outcome, and evidence location
AND they MUST NOT claim distributed evaluation, evaluator equivalence, remote-build correctness, streaming feasibility, or release eligibility beyond the recorded evidence.
