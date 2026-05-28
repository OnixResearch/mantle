## ADDED Requirements

### Requirement: Native target proc-macro host extern binding

r[rust_package_planning.native_target_proc_macro_host_extern_binding] Mantle MUST bind planned proc-macro host artifacts into target rustc extern arguments before invoking target rustc.

#### Scenario: Target receives proc-macro extern from consumed host artifact

GIVEN a native target unit consumes a proc-macro host artifact through `consumed_host_artifacts`
AND the target unit has no matching dependency artifact placeholder for that proc-macro package
WHEN Mantle prepares the target unit for rustc execution
THEN Mantle MUST add a deterministic `--extern <proc-macro-crate>=<produced-host-artifact>` argument.
AND Mantle MUST bind the produced host artifact path into the target host-artifact receipt material.

#### Scenario: Existing dependency placeholder is not duplicated

GIVEN a native target unit consumes a proc-macro host artifact
AND the target unit already has a matching dependency artifact placeholder
WHEN Mantle prepares the target unit for rustc execution
THEN Mantle MUST rewrite the placeholder to the produced host artifact path.
AND Mantle MUST NOT add a duplicate `--extern` argument for the same proc-macro crate.

#### Scenario: Missing host artifact remains fail-closed

GIVEN a native target unit consumes a proc-macro host artifact
WHEN the corresponding host producer has not produced an artifact path
THEN Mantle MUST fail before invoking target rustc with deterministic missing-host-artifact diagnostics.
AND Mantle MUST NOT search the sysroot, Cargo target directories, or ambient caches to repair the missing proc-macro material.

#### Scenario: Darling proc-macro frontier moves

GIVEN topology execution currently blocks while compiling `darling@0.20.11` because `extern crate darling_macro` resolves from the sysroot
WHEN native target proc-macro host extern binding is applied
THEN self-probe verification MUST show that this sysroot `darling_macro` blocker no longer stops the topology at `darling@0.20.11`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
