# Design: Thin the CLI composition root

## Context

The Mantle binary is the primary composition root. It currently also contains application policy and capability orchestration. This change establishes a cross-cutting application architecture after the store, remote, Rust-plan, and build-plan boundaries exist.

The required command flow is:

```text
argv
  -> CLI inbound adapter
  -> application command
  -> capability-scoped application operation
  -> functional core decisions and effect plans
  -> application operation executes ports
  -> application result
  -> human or JSON presentation adapter
  -> exit status
```

## Decisions

### Decision: keep the root dispatcher mechanical

`main.rs` will parse global options, build the runtime context, select concrete adapters, dispatch one application operation, render one result, and select the exit status.

It will not own domain validation, route policy, retry policy, trust policy, state transitions, receipt construction, filesystem traversal, process behavior, or provider translation.

**Rationale:** The composition root wires behavior. It does not define domain meaning.

### Decision: use capability-scoped application operations

Each command family will expose an application command and result using Mantle-owned types. An operation can coordinate core decisions and genuine external ports for one coherent capability.

There will be no global service container. Dependencies will be explicit constructor fields or parameters at the visible composition root.

### Decision: separate inbound and presentation adapters

Clap records remain inbound CLI DTOs. Command adapters will validate syntax and map DTOs into application commands. Human and JSON output modules will render application results without changing their meaning.

Accepted command names, options, JSON fields, diagnostics, and exit codes will remain stable.

### Decision: map errors only at the outer boundary

Cores return typed domain blockers. Application ports return capability errors. Adapters retain provider errors. The CLI boundary maps these types to the stable `RunError` envelope and exit status.

String diagnostics will not replace typed state inside cores or port contracts.

### Decision: keep effect execution visible

Application operations will receive core effect plans, invoke explicit ports, and record typed observations. A plan, dispatch, or successful port call will not become a stronger claim than the returned observation supports.

### Decision: add one maintained architecture rail

A deterministic Rust checker will inspect package dependencies, forbidden core imports, port ownership, broad infrastructure types, root dispatcher size and allowlisted responsibilities, and presentation dependencies.

Positive fixtures will prove accepted dependency direction. Negative fixtures will prove that a core importing CLI, Snix, filesystem, process, async, clock, random, or network authority fails the rail.

## Migration

Command families will move one at a time. Each move will preserve command and output golden tests. The root dispatcher will retain compatibility delegation only while a family migrates. New policy must not enter the compatibility path.

## Testing and evidence

Tests will cover valid and invalid CLI mapping, missing dependencies, adapter faults, domain blockers, JSON and human parity, exit-code parity, effect failures, and architecture negatives.

Evidence proves source topology, deterministic core decisions, and bounded observed adapter results. It does not prove external effect success without observation, provider correctness, deployment success, or release eligibility.

## Risks

- Application operations can become new monoliths. Each operation must own one coherent capability.
- A dependency container can hide authority. Dependencies must remain explicit and narrow.
- Output parity can force domain strings into cores. Presentation compatibility must stay in adapters.
