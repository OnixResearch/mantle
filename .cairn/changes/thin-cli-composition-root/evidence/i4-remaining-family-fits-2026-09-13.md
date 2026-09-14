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

## Follow-up: what today's CLI can and cannot populate

Measured after mapping filegen and the named shell roots.

**Remote execution.** `remote-secret-worker` maps cleanly: it names its manifest,
profile, and provider, and the profile is exactly the credential the contract's
`SecretProfile` operation requires. The four visible actions do not map, and the
reason is structural rather than cosmetic: the contract's `Build` and `Fetch`
operations require declared entries, while `remote serve` carries its already
present input refs as an optional repeatable flag and `ticket`, `status`, and
`debug` name no build inputs at all. Those actions therefore stay outside the
contract rather than declaring entries the operator never wrote.

**Diagnostics.** Neither CLI root fits the operation set:

| Root | CLI shape | Contract requirement |
| --- | --- | --- |
| `doctor` | `doctor --profile <build\|self-build>` | operations are `Trace`, `Lint`, `Refactor`, `Bench`; a workflow profile is not one |
| `refactor` | `refactor {list, plan, check, apply}` over a session id, project root, and repeatable store prefixes | `Lint` and `Refactor` require declared entries, which the contract reads as rule names; a session id is a subject, not a rule |

Measured field detail: `RefactorAction::Plan`, `Check`, and `Apply` carry
`session`, `root`, and `store_prefixes`, and `List` carries nothing. A mapping
would have to either treat the session as a rule name or reject a refactor that
declares no store prefix, and both would change what the command means.

## Decision

- Map `remote-secret-worker` to `SecretProfile`; leave the four visible remote
  actions unadministered.
- Leave both diagnostics roots unadministered until the family contract is
  revised to describe the CLI's surface (a profile for the doctor, a session for
  the refactor) or the CLI grows trace, lint, and bench roots.

## Non-claims

Leaving a root unadministered is not a claim that the root is wrong; it is a
claim that the contract cannot check it yet.
