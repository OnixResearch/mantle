# I4 remaining family fits

Change: `thin-cli-composition-root`
Task-ID: I4
Subject revision: `d14eda707` plus this prose-only evidence commit.

## Question

Five families still have no CLI consumer of their contract command. Which of
them can be mapped as-is, and which need a contract decision first?

## Inspected evidence

Contract operation sets against the CLI subcommands at the recorded revision.

| Family | Contract operations | CLI surface | Fit |
| --- | --- | --- | --- |
| Realization (`RealizeCommand`) | one command, not an operation set: `roots`, `profile`, `requested_jobs`, `dry_run` | `build` carries `name`, `jobs`, `plan` | `roots`, `requested_jobs`, and `dry_run` fit; `profile` does not |
| Source provenance | Admit, Bundle, Hydrate, Attest, Export | `source bundle {plan, export, bootstrap-profile, refresh, list, import, hydrate-self-build, verify, preflight}` | closest fit: bundle, export, and hydrate-self-build map; Admit, Attest, and a verify operation have no operation of their own |
| Remote execution | Build, Stage, Fetch, Doctor, SecretProfile | `remote {ticket, status, debug, serve}`, `remote-secret-worker` | no overlap between the operation set and the subcommand set |
| Component flow | Build, Bundle, Verify | `wasm-component {build}` | build maps; bundle and verify have no CLI subcommand |
| Diagnostics | Trace, Lint, Refactor, Bench | `doctor` takes `DoctorProfile {build, self-build}`, `refactor` takes `RefactorAction` | recorded earlier: the doctor profile is not a diagnostics operation |

Measured detail on the realization gap: `BuildProfile` appears **zero** times in
`src/`, so no CLI path produces the profile the realization command requires.
The build DTO's nearest facts are `--strict-hermetic` and `--impure`, which are
a hermeticity mode rather than a dev/release build profile.

## Decision

Do not map a family by inventing the missing field. Each remaining family needs
one of three decisions, and the decision belongs to whoever owns the family
contract:

1. **Realization**: either make `profile` optional (the operator did not declare
   one), or replace it with the hermeticity mode the CLI actually carries.
2. **Source provenance**: add the operations the CLI already has (`Verify`), or
   map the bundle subcommands onto the existing five.
3. **Component flow**: map `build` now and add the bundle and verify operations
   when those CLI subcommands exist.
4. **Remote execution and diagnostics**: the operation sets describe a different
   surface than the CLI exposes, so the contract needs a deliberate revision
   rather than a mapping.

## Owner

The family shapes live in `crates/mantle-application-contract`; the CLI roots
live in `src/main.rs`. The DTO mapping stays in `src/command_input.rs`.

## Next action

Take source provenance first: it is the smallest gap (one missing operation, or
a bundle-subcommand mapping), and it already has a `SourceProvenanceCommand`
with subject, declared entries, and source path.

## Non-claims

This note measures fit. It does not claim the CLI or the contract is wrong, that
any family should change behaviour, or that I4 is complete.
