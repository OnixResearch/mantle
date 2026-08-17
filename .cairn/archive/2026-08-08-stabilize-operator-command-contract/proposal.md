# Stabilize the Mantle operator command contract

## Why

Mantle has a canonical product name, a naming drift guard, broad operator documentation, and stable diagnostics for selected workflows. It does not yet have one machine-checked inventory that connects CLI commands, aliases, project files, environment inputs, machine schemas, help text, exit behavior, and remediation commands.

The gap lets command help, documentation, error suggestions, and compatibility surfaces drift independently. Some current diagnostics still suggest legacy commands even when the operator invoked `mantle`. A new compatibility token can also become permanent without an owner, migration state, or removal condition.

Mantle needs one operator contract that makes the supported daily workflow discoverable and keeps compatibility explicit without renaming exact identifiers by accident.

## What Changes

- Add a typed Nickel inventory for canonical commands, compatibility aliases, project files, environment inputs, machine schemas, support tiers, and migration states.
- Export a deterministic command catalog from the Clap command graph and compare it with the reviewed inventory.
- Generate the command reference and canonical quick workflow from the checked catalog instead of maintaining independent command lists.
- Extend stable diagnostics with bounded structured remediation actions that name side effects, network needs, and preconditions.
- Reject new undocumented legacy identifiers and stale operator commands while preserving exact existing compatibility identifiers until their own migration is accepted.
- Add positive and negative drift fixtures for help, docs, JSON output, exit status, remediation, aliases, and secret redaction.

## Non-Goals

- Renaming existing `crunch-*` crates, APIs, paths, sidecars, or serialized schemas in this change.
- Removing a compatibility surface without a separate migration and consumer evidence.
- Treating generated command documentation as build, security, or release evidence.
- Moving Onix system, service, or deployment workflows into Mantle.

## Impact

- **Affected specs:** `operator-diagnostics`
- **Planned files:** typed Nickel operator inventory, generated runtime data, Clap catalog adapter, diagnostic core and renderers, command-reference generator, drift checks, fixtures, and operator docs
- **Compatibility:** exact legacy identifiers remain readable according to their recorded state; new operator prose and remediation use Mantle commands
- **Testing:** deterministic catalog checks, positive canonical workflows, negative drift and secret fixtures, machine-output checks, and Cairn gates
