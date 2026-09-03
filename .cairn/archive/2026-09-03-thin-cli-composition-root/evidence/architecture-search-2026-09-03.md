# Architecture search — 2026-09-03

## Question

Can Mantle reuse an existing OnixResearch application-boundary component for its CLI root?

## Inspected evidence

- `OnixResearch/animus` at `ce968741e2761827eb5d5046854ff991af0fb9f9`:
  - `docs/safety-contract.md`;
  - `docs/live-steel-agent-shell.md`;
  - `crates/animus-std-eval/src/host.rs`;
  - repository Rust ports found with `pub trait .*Port`.
- `OnixResearch/lattice` at `03dfa18992f1c0c9bc79bb981a407f97cc474177`:
  - `src/agent_evaluation/runtime/runner.rs`;
  - repository Rust ports found with `pub trait .*Port`.
- A workspace search for `ApplicationCommand`, `ApplicationEffect`, `ApplicationPortError`, `application-owned port`, and `composition root`.

Animus supplies a useful pattern: no-std contracts, separate narrow ports, a visible composition root, and a host shell. Its ports model agent evaluation and host services, not Mantle command families.

Lattice supplies explicit effect ports, but its `Services` aggregate is a broad service container. That shape conflicts with this change's explicit ban on a global service container.

No inspected component owns Mantle's command-family semantics, stable CLI errors, or existing operation compatibility boundary.

## Decision

Do not copy or depend on an existing component. Add Mantle-owned `mantle-application-core` and `mantle-application` components.

Use the shared architectural pattern only:

- deterministic no-std command and effect types;
- application-owned narrow ports;
- explicit outer adapters;
- typed observations before terminal outcomes.

Keep the 12 Mantle command-family ports separate. Do not add a Lattice-style `Services` record or a generic repository abstraction.

## Owner

Mantle owns command admission, CLI compatibility, command-family ports, and the final presentation boundary.

## Next action

Keep provider types in outer adapters. Run the maintained architecture rail and all adversarial fixtures before lifecycle acceptance.
