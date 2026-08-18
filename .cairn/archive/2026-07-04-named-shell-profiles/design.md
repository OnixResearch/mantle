# Design: Named shell profiles

## Architecture

Named shell profiles extend the existing shell adapter with project-level selection data.

- Pure core: validates profile names, default resolution, environment entries, path entries, hook declarations, profile inheritance or merge rules if supported, and non-claim diagnostics. It returns an activation plan over owned strings and bounded lists.
- Imperative shell: resolves built package outputs, reads sidecar JSON, maps UTF-8 paths to host `PathBuf`/`OsString`, and executes or prints activation instructions.

Profiles lower to concrete build inputs or shell sidecars before execution. Mantle core does not interpret frontend package/module semantics to fill a profile. Shell profiles are not the source of truth for package action specs; package/build declarations remain the source for build identity.

## Profile model

Initial conventions:

- `build`: convenience profile that mirrors the minimal tools an operator needs to build or inspect the project, but does not define package action identity.
- `dev`: richer operator convenience environment that must be non-mutating by default and decoupled from build hashes, file generation, lock refresh, and release evidence.
- `default`: alias to `dev` unless explicitly set.

A project may add other named profiles if they satisfy the same validation rules. Invalid names, ambiguous defaults, duplicate env keys after normalization, unsupported path entries, and non-UTF-8 adapter inputs should fail before activation.

## Decoupling model

Shell activation is a convenience workflow, not build evidence. `mantle shell` may build declared shell inputs before activation, but those shell inputs do not become package action inputs unless the package declaration references them independently. Entering a shell must not regenerate files, mutate lockfiles, refresh inputs, change project manifests, or alter store proof state except for explicitly building requested shell inputs under ordinary build commands.

If shell activation needs evidence, Mantle should emit a separate shell-activation receipt that binds the selected profile, activation plan, and applied environment/path changes. Build reports, release evidence, and reproducibility claims must not cite shell activation as proof unless that separate receipt is explicitly in scope and the claim is limited to activation.

## Services boundary

Service declarations are intentionally out of scope. If a frontend wants Organist-like services, it can generate opaque artifacts or invoke its own service manager outside Mantle. Mantle may later build service descriptors as data, but it must not own process lifecycle semantics in this profile change.

## Validation strategy

Positive tests should cover default resolution, explicit profile selection, `build` versus `dev` differences, env/PATH activation planning, JSON sidecar compatibility, and proof that shell activation leaves build action identity unchanged. Negative tests should cover missing profiles, invalid names, ambiguous defaults, duplicate env entries, unsupported service declarations, non-UTF-8 paths at the adapter boundary, attempted filegen/lock mutation during shell activation, and overclaiming diagnostics.
