# Remote Builds Specification

## Purpose

Define policy-safe failed-sandbox capture for remote debugging.

## Requirements

### Requirement: Failed remote sandbox capture is explicit and bounded [r[remote_builds.failure_debug_capture]]

Mantle MUST default remote failure bundles to metadata-only evidence. Failed-sandbox artifact capture MUST require explicit typed policy defining allowed relative paths or artifact classes, sensitivity policy, regular-file handling, maximum files/bytes/depth, retention, and cleanup behavior. Capture selection MUST be decided before filesystem reads, and rejected capture MUST NOT delay unboundedly or rewrite the remote build result.

#### Scenario: Allowed artifact is captured before cleanup

- GIVEN a remote attempt fails and policy explicitly allows a bounded regular-file artifact under the sandbox root
- WHEN Mantle applies the accepted capture plan before cleanup
- THEN it MAY ingest the artifact as a content-addressed debug object and bind its ref in the bundle
- AND sandbox cleanup or quarantine MUST continue according to the recorded policy outcome.

#### Scenario: Unsafe artifact is rejected

- GIVEN a requested artifact is absolute, traverses above the sandbox, follows an escaping symlink, is a socket/device/FIFO, exceeds file/count/byte/depth limits, or matches sensitive policy
- WHEN capture planning or application reaches it
- THEN Mantle MUST reject the artifact before publication
- AND it MUST NOT include host content, secrets, or an unverified ref in the debug bundle.

#### Scenario: Capture failure does not rewrite build truth

- GIVEN the remote build has already failed and debug capture encounters an I/O, quota, scrub, ingestion, or cleanup error
- WHEN Mantle reports the attempt
- THEN the original failure phase and output-admission state MUST remain unchanged
- AND debug-capture degradation or quarantine MUST be reported as a separate diagnostic fact.

#### Scenario: Retention preserves active replay lease only

- GIVEN debug bundles and captured objects are subject to retention while one bundle has an active inspect or replay lease
- WHEN retention runs
- THEN Mantle MUST preserve the active leased bundle and apply only the accepted bounded deletion plan to eligible roots
- AND expired metadata MUST NOT authorize deletion of ordinary build outputs or unrelated CAS objects.
