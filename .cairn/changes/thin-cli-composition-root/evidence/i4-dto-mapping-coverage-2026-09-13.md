# I4 DTO mapping coverage

Change: `thin-cli-composition-root`
Task-ID: I4
Subject revision: `9c29375cc` plus this prose-only evidence commit.

## Question

I4 asks for the Clap DTO mapping to be split from human and JSON presentation.
The presentation half is done. This note measures how much of the mapping half
exists, so the remainder is legible instead of assumed.

## Inspected evidence

Every contract family declares a command type. Counting the CLI consumers of
each command type at the recorded revision:

| Contract command type | CLI consumers |
| --- | --- |
| `StoreAdministrationCommand` | `src/command_input.rs` (mapped, contract-gated) |
| `ReleaseCommand` | `src/command_input.rs` (mapped, contract-gated) |
| `RealizeCommand` | none — `src/foreign_import_cmd.rs` defines its own request struct |
| `BootstrapCommand` | none — `src/main.rs` defines `BootstrapCommandRequest` |
| `SourceProvenanceCommand` | none |
| `PlanningCommand` | none |
| `EvaluationCommand` | none |
| `RemoteExecutionCommand` | none |
| `ProjectCommand` | none |
| `ComponentCommand` | none |
| `DiagnosticsCommand` | none |

Contract imports across the binary live in four files: `src/main.rs` (50),
`src/store_cmd.rs` (8), `src/run_binary_selection.rs` (8), and
`src/command_input.rs` (6).

Two shape mismatches show why the remainder is not a mechanical move:

- The doctor command takes a `DoctorProfile` (`build`, `self-build`), while the
  contract's `DiagnosticsOperation` models `trace`, `lint`, `refactor`, and
  `bench`. Mapping doctor onto those operations would invent meaning.
- The bootstrap CLI exposes `capabilities`, `parity-report`,
  `rust-source-provider`, `full-source-*`, `native-toolchain-closure`, and
  `validate`, while the contract's `BootstrapOperation` models `fetch`,
  `self-build`, `inventory`, and `seed-reduce`.

## Decision

I4 remains open. Its remaining work is per family, in this order:

1. Fit the contract command to the CLI DTO where the shapes disagree (doctor
   profile, bootstrap subcommands, project operations).
2. Map the fitted DTO in `src/command_input.rs` with a total match, so the
   compiler catches an unclassified variant as it did for
   `StoreAction::Composition`.
3. Gate modeled requests through the contract before any effect, as the store
   and release families already do.

## Owner

The DTO mapping lives in `src/command_input.rs`; the family shapes live in
`crates/mantle-application-contract`.

## Next action

Fit and map the project-lifecycle family next: it has six operations, the
contract command already carries a subject and manifest path, and no CLI
consumer exists yet.

## Non-claims

This note measures mapping coverage. It does not claim the unmapped families are
wrong, that their CLI behaviour should change, or that I4 is complete.
