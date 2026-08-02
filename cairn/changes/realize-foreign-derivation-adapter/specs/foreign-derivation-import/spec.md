## ADDED Requirements

### Requirement: Receipt-bound foreign source materialization

r[foreign_derivation_import.source_materialization] Mantle MUST materialize every foreign non-derivation source from a verified Mantle source record or an admitted fixed-output fetch unit. Realization MUST NOT read source bytes from an ambient foreign store path.

#### Scenario: Source bundle record becomes a target source

GIVEN an executable plan names a source requirement with an accepted source-bundle record and expected identities
WHEN the realization adapter prepares native units
THEN it MUST verify and materialize the record through Mantle castore and PathInfo services
AND it MUST record the exact target source path before any parent uses that path.

#### Scenario: Fixed-output node uses ordered candidates

GIVEN a compiled fixed-output node declares bounded ordered source candidates and one required content identity
WHEN Mantle attempts acquisition
THEN it MUST preserve candidate order and verify the required content identity
AND a content mismatch MUST fail closed without PathInfo admission.

#### Scenario: Missing or changed source blocks realization

GIVEN a required source record is missing, unsigned, incomplete, digest-mismatched, mode-mismatched, or outside the admitted bundle
WHEN realization validates sources
THEN it MUST fail before dependent builder dispatch
AND it MUST NOT fall back to an ambient Guix, Nix, or host store path.

### Requirement: Per-derivation foreign execution profiles

r[foreign_derivation_import.execution_profile] Mantle MUST apply a typed, explicit execution profile to every realized foreign derivation. The profile digest MUST contribute to target derivation identity and MUST bind all compatibility behavior used during execution.

#### Scenario: Guix profile omits ambient shell

GIVEN a foreign node selects the admitted Guix execution profile
WHEN Mantle builds its request
THEN `provide_bin_sh` MUST be false and `/bin/sh` MUST not appear unless it is an explicitly mapped declared input
AND Mantle MUST NOT inject Nix-style shell compatibility through a global default.

#### Scenario: Profile controls the execution boundary

GIVEN a profile declares environment mode, work directory, network policy, setid chmod policy, syscall exceptions, writable prefixes, and resource limits
WHEN the build request is created
THEN each declared field MUST control the corresponding request behavior
AND undeclared compatibility behavior MUST fail closed.

#### Scenario: Profile policy changes target identity

GIVEN two otherwise identical target derivations select different canonical execution profiles
WHEN target identities are computed
THEN their derivation identities MUST differ
AND the builder MUST reject a profile whose BLAKE3 does not match the identity-bound profile digest.

#### Scenario: Reserved profile field is protected

GIVEN a foreign derivation already declares Mantle’s reserved profile binding field
WHEN graph compilation or realization validates the node
THEN it MUST reject the node with a deterministic reserved-field diagnostic
AND it MUST NOT let builder-controlled input override execution policy.

### Requirement: Thin Mantle foreign realization adapter

r[foreign_derivation_import.realization_adapter] Mantle MUST realize accepted executable foreign plans through its ordinary derivation registry, scheduler, build services, and store. The adapter MUST NOT add foreign frontend semantics or a second execution engine.

#### Scenario: Selected roots use the ordinary worker

GIVEN an admitted executable plan, verified sources, and accepted execution profiles
WHEN an operator selects local realization
THEN the adapter MUST register resolved native units and call the ordinary `Builder` worker path
AND normal goal ordering, substitution, fetch, sandbox, cancellation, PathInfo, and attestation behavior MUST remain in force.

#### Scenario: Two-node graph realizes exact dependency

GIVEN a compiled child and parent use an exact target output mapping
WHEN Mantle realizes the parent root
THEN the child MUST complete before the parent becomes ready
AND the parent builder MUST receive the exact child output path recorded in the executable plan.

#### Scenario: Partial selected-root failure is visible

GIVEN one selected root succeeds and one selected root fails
WHEN the worker completes the selected set
THEN the report MUST preserve each root outcome and the successful store facts
AND the command MUST return failure without claiming complete realization.

#### Scenario: Foreign frontend is absent during realization

GIVEN accepted artifacts were produced on another host
WHEN Mantle realizes them locally
THEN it MUST not execute Guix, Nix, `nix-store`, flakes, overlays, Scheme evaluation, or package-module evaluation
AND missing foreign frontend binaries MUST not affect the result.

#### Scenario: Unsupported remote route fails early

GIVEN the first adapter version receives a remote-builder request
WHEN route validation runs
THEN it MUST fail before sending source, profile, derivation, or credential data
AND it MUST identify remote foreign realization as unsupported.

### Requirement: Separate foreign realization receipt

r[foreign_derivation_import.realization_receipt] Mantle MUST emit `mantle-foreign-realization-receipt-v1` separately from import receipts and ordinary build reports. The receipt MUST bind the exact artifacts and observations used for one realization attempt.

#### Scenario: Successful local realization is reviewable

GIVEN all selected roots finish through the ordinary worker
WHEN Mantle writes the realization receipt
THEN it MUST bind import receipt, executable plan, source records, execution policy, selected roots, build-report digest, PathInfo identities, and action dispositions
AND it MUST identify whether each output was fetched, substituted, built, or already present.

#### Scenario: Preflight rejection emits no realization receipt

GIVEN source, profile, root, route, or artifact preflight rejects the request
WHEN Mantle reports the rejection
THEN it MUST NOT create a realization receipt or realization store state
AND it MUST identify the stable preflight failure.

#### Scenario: Execution failure emits bounded evidence without success

GIVEN execution has started and dispatch, build, cancellation, persistence, or report writing fails
WHEN Mantle reports the attempt
THEN diagnostics MUST identify the stable failure stage and affected roots
AND no receipt may report a stronger state than the completed evidence supports.

#### Scenario: Realization preserves system boundary and non-claims

GIVEN a foreign package graph realizes successfully
WHEN an operator reviews the receipt and documentation
THEN output provenance, package correctness, bootstrap parity, reproducibility, OS activation, account setup, initrd construction, VM boot, and deployment MUST remain separate claims
AND OnixOS MUST remain responsible for system assembly and boot evidence.
