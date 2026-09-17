# Specification: Build authority attenuation

## ADDED Requirements

### Requirement: Caveat filters rewrite, reject, or reject everything

r[mantle.authority_attenuation.caveat_filter_semantics] Mantle MUST define a
bounded caveat filter language over assertion and message values. A filter MUST
be one of: a pattern rewrite with a template, a pattern reject, an alternative
list of rewrites, or an unknown caveat that rejects every input.

A rejected assertion or message MUST be discarded without protocol-level
feedback. An oversized, malformed, or unsupported caveat MUST fail closed.

#### Scenario: Rewrite with bindings

- GIVEN a caveat that rewrites a record matching a pattern into a namespaced
  record
- WHEN a matching assertion arrives
- THEN the rewritten value MUST be forwarded
- AND a non-matching assertion MUST be discarded silently

#### Scenario: Unknown caveat rejects everything

- GIVEN a caveat whose variant is not in the accepted set
- WHEN any assertion or message arrives
- THEN it MUST be rejected
- AND no feedback MUST be sent to the sender

### Requirement: Composition cannot widen a grant

r[mantle.authority_attenuation.composition_order] Caveats MUST apply right to
left. Composing a caveat chain onto an existing grant MUST NOT widen the set of
accepted inputs. Chain length and pattern size MUST be bounded, and a bound
violation MUST fail closed.

#### Scenario: Composed chain is no wider

- GIVEN a grant that accepts input set `S`
- WHEN a further caveat chain is composed onto it
- THEN the accepted input set MUST be a subset of `S`

### Requirement: Attenuated grants restrict remote work and store views

r[mantle.authority_attenuation.attenuated_remote_grant] A remote builder grant
MAY be restricted to a declared job set, output class, and deadline. The
receiver MUST check the restriction before admitting a job. A project
configuration capability MAY be restricted so its declarations are rewritten
into project-namespaced goals.

The coordination daemon from ADR 0080 MAY issue grants. Receivers MUST enforce
them locally and MUST NOT require a daemon round trip for admission.

#### Scenario: Restricted grant refuses a second job

- GIVEN a remote builder grant restricted to one declared job
- WHEN a second job is submitted under the same grant
- THEN the receiver MUST refuse it
- AND MUST NOT start work or expose any reason to the sender

#### Scenario: Project isolation

- GIVEN a project capability with rewritten project-namespaced goals
- WHEN the project requests another project's goal
- THEN the request MUST be discarded silently

### Requirement: Store views are restricted by pattern

r[mantle.authority_attenuation.attenuated_store_view] A store view MAY be
restricted to a declared logical-path pattern. A read outside the pattern MUST
be refused, and the refusal MUST NOT expose paths outside the view.

#### Scenario: View escape refused

- GIVEN a store view restricted to one logical-path pattern
- WHEN a caller requests a path outside the pattern
- THEN the request MUST be refused with no path disclosure

### Requirement: Stack authority components are evaluated first

r[mantle.authority_attenuation.stack_authority_reuse] Before Mantle defines any
new token construction, it MUST evaluate whether the stack's UCAN component,
under Basalt policy, can carry the required pattern caveats, composition, and
revocation. The evaluation decision MUST be recorded in an ADR.

#### Scenario: Decision recorded before implementation

- GIVEN the stack authority evaluation
- WHEN a token construction is chosen
- THEN an ADR MUST record the chosen component, the rejected alternatives, and
  the claim boundary
