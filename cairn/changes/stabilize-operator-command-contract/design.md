# Design: Stable Mantle operator command contract

## Context

`docs/mantle-naming.md` classifies stale prose and exact compatibility identifiers. `scripts/check-stale-branding.rs` enforces that prose boundary. Clap owns the executable command graph. README and `docs/operator-workflows.md` describe common workflows. Diagnostics in `src/errors.rs` still derive some suggestions from error strings, and no single artifact proves that commands, docs, aliases, schemas, and remediation agree.

This change adds a product contract above those existing parts. It does not perform a broad rename.

## Decisions

### Decision: Use a typed Nickel inventory as the reviewed policy source

**Choice:** Add a typed Nickel inventory with one row per operator surface. Each row names its kind, canonical spelling, compatibility spellings, owner, support tier, mutation class, network class, machine-output schema when applicable, migration state, and removal gate.

The shell evaluates the Nickel source during generation. Runtime code consumes checked-in deterministic generated data and does not execute Nickel for ordinary diagnostics.

**Rationale:** Compatibility and support choices are configuration policy. Nickel contracts make missing owners, unknown states, duplicate spellings, and conflicting command roles fail before publication.

### Decision: Derive executable command facts from Clap

**Choice:** A thin shell walks the Clap command graph and produces normalized in-memory descriptors. A pure core validates those descriptors against the reviewed inventory and creates an ordered command catalog.

The core receives command paths, aliases, flags, value requirements, conflicts, help summaries, and machine-output support as plain values. It performs no filesystem, environment, process, or rendering work.

**Rationale:** Clap is authoritative for parser behavior. The Nickel inventory is authoritative for product policy. Comparing both catches drift without duplicating parser logic in configuration.

### Decision: Generate command reference text from the accepted catalog

**Choice:** Generate the command reference and one canonical daily workflow from the accepted catalog. README keeps a short hand-written product introduction and links to generated command material.

The workflow covers `mantle doctor`, project inspection or refresh, plan, build or remote build, and evidence inspection. Each step states whether it can mutate files, store state, or contact the network.

**Rationale:** Users need one small path. Generated details reduce stale flag and command examples while keeping product explanation readable.

### Decision: Represent remediation as structured data

**Choice:** Stable diagnostics use a versioned remediation record with a diagnostic code, phase, safe subject, message, evidence references, and ordered next actions. Each next action records a Mantle command path, mutation class, network class, required preconditions, and a bounded explanation.

A pure classifier maps normalized failure facts to remediation records. Human and JSON renderers consume the completed record. Raw errors can remain supplemental bounded context but cannot invent command suggestions during rendering.

**Rationale:** A command suggestion is operational behavior. Structured actions make side effects visible and let tests prove that JSON and human output agree.

### Decision: Preserve compatibility through explicit states

**Choice:** Existing exact compatibility identifiers start as recorded compatibility rows. Supported states are canonical, compatibility-read-write, compatibility-read-only, and historical-only.

A transition to a narrower state requires consumer inventory, positive legacy-read evidence where promised, negative write rejection where required, migration instructions, and rollback policy. This change does not perform those transitions.

**Rationale:** Immediate renaming would break machine contracts. Unbounded compatibility would preserve product confusion forever.

### Decision: Keep stdout, exit, and secret rules explicit

**Choice:** Every cataloged command records its success and policy-rejection exit classes plus its stdout contract. JSON mode emits one documented JSON value when the command contract promises JSON. Diagnostics and remediation stay redacted and outside JSON stdout.

**Rationale:** Documentation parity is incomplete when automation can still observe unstable streams or exit behavior.

## Validation

Positive fixtures cover the canonical workflow, canonical command help, compatibility reads, deterministic catalog ordering, human remediation, JSON remediation, and generated-doc parity.

Negative fixtures cover undocumented commands, stale docs, unowned aliases, duplicate spellings, unknown schemas, incorrect mutation labels, legacy suggestions in canonical diagnostics, secret-bearing arguments, malformed generated data, and stdout pollution.

## Risks / Trade-offs

- The inventory adds review work whenever the CLI changes.
- Generated documentation can become noisy if it mirrors every internal option without support-tier filtering.
- Some legacy identifiers will remain visible until separate migrations prove safe narrowing.
- Stable remediation codes create a compatibility surface and require deliberate versioning.
