## ADDED Requirements

### Requirement: Builder protocols remain at compatibility edges [r[build_tool_boundary.builder_protocol_edge]]

Mantle MUST express object admission, execution-unit admission, declared-result binding, and child-authority attenuation with Mantle-owned contracts. Nix daemon operations, Varlink messages, file-descriptor transport, and builder-RPC feature types MUST remain inside declared compatibility adapters and MUST NOT define Mantle's native build API.

#### Scenario: A future Nix adapter maps a builder operation

- GIVEN an accepted consumer requires a supported Nix builder-facing operation
- WHEN a Nix adapter translates the operation
- THEN it MUST map the request into existing Mantle-owned capabilities and typed authority facts
- AND Mantle's core MUST remain independent of the protocol transport and version

#### Scenario: A provider type leaks into a core contract

- GIVEN a core command, result, blocker, effect plan, port, receipt, or public native plan exposes a Nix daemon, Varlink, descriptor-passing, or builder-RPC type
- WHEN architecture validation runs
- THEN it MUST fail with a deterministic provider-leak diagnostic
- AND translation MUST move to the declared compatibility adapter

#### Scenario: A non-Nix frontend creates dynamic work

- GIVEN a frontend emits a valid native dynamic plan with Mantle-owned authority facts
- WHEN Mantle admits and schedules the plan
- THEN the frontend MUST NOT need Nix store operations, Nix daemon negotiation, Varlink, or `builder-rpc-v0`
- AND the same child-authority attenuation policy MUST apply

#### Scenario: Draft protocol work has no current consumer

- GIVEN an upstream builder protocol remains experimental and no admitted Mantle consumer requires it
- WHEN roadmap scope is selected
- THEN Mantle MUST retain explicit unsupported classification instead of adding a speculative core abstraction
- AND later protocol adoption MUST require a separate consumer-driven Cairn change
