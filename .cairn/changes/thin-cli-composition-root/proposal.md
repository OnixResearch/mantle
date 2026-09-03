# Change: Thin the CLI composition root

## Why

`src/main.rs` currently parses arguments and selects adapters, but it also owns substantial application policy, runtime fingerprinting, replay orchestration, command classification, and error translation. Several application ports return CLI-owned `RunError` values, which points dependencies outward.

A visible composition root must wire capabilities and dispatch application operations. Domain decisions, effect planning, adapter execution, and presentation must remain separate.

## What Changes

- Define an accepted application-architecture contract for Mantle commands.
- Reduce `main.rs` to argument parsing, runtime-context construction, adapter selection, application dispatch, and final presentation.
- Add capability-scoped application operations with Mantle-owned commands, results, blockers, and port contracts.
- Keep CLI DTO translation in inbound adapters and human or JSON rendering in outbound presentation adapters.
- Map typed domain and capability errors into `RunError` only at the CLI boundary.
- Select concrete filesystem, process, network, store, clock, random, credential, and telemetry adapters at visible composition roots.
- Add dependency and source guards that reject domain policy, raw infrastructure types, or effect execution in the root dispatcher.
- Preserve command names, arguments, exit codes, JSON, diagnostics, and mutation behavior.

## Non-Goals

- Creating one global service container or generic repository abstraction.
- Adding ports for pure internal functions.
- Changing accepted command behavior or public output schemas.
- Moving provider or framework types into functional cores.
- Treating dispatch or an effect plan as proof that an external effect succeeded.

## Dependencies

- `complete-store-capability-migration`
- `separate-remote-build-hexagon`
- `separate-rust-plan-hexagon`
- `extract-build-planning-core`

These changes establish the application operations and narrow capabilities that the final CLI root will compose.

## Impact

- **Affected spec:** new `application-architecture` capability
- **Affected code:** `src/main.rs`, command modules, application operations, port contracts, error mapping, presentation adapters, and architecture checks
- **Compatibility:** accepted CLI syntax, JSON, exit codes, diagnostics, and mutation behavior remain unchanged
- **Testing:** command parity tests, positive and negative adapter tests, error mapping tests, dependency guards, source guards, and Cairn gates
