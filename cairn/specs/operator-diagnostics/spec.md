# Operator Diagnostics Specification

## Purpose

Defines the `operator-diagnostics` capability.

## Requirements

### Requirement: Mantle emits a verbose runtime fingerprint [r[operator_diagnostics.verbose_runtime_fingerprint]]

Mantle MUST emit a concise runtime fingerprint when an operator explicitly requests verbose or talkative diagnostics. The fingerprint MUST identify the Mantle binary version, sanitized command label, logical store prefix, physical store directory, state directory, JSON mode, verbosity source, and command-specific mode fields such as hermeticity or substitution mode when those fields affect behavior. The fingerprint MUST be diagnostic context only and MUST NOT be presented as proof of build success or correctness.

#### Scenario: Verbose command prints context [r[operator_diagnostics.verbose_runtime_fingerprint.scenario.verbose]]

- GIVEN an operator invokes a Mantle command with explicit verbose or talkative diagnostics enabled
- WHEN Mantle starts command execution
- THEN Mantle MUST emit a runtime fingerprint before long-running or mutating work begins
- AND the fingerprint MUST include version, command label, logical store prefix, physical store directory, and state directory.

#### Scenario: Command-specific modes are included when applicable [r[operator_diagnostics.verbose_runtime_fingerprint.scenario.command-modes]]

- GIVEN a command has behavior affected by hermeticity, substitution, trust, or Nix-compatibility mode
- WHEN Mantle emits the verbose runtime fingerprint for that command
- THEN the fingerprint MUST include stable redacted fields describing those selected modes
- AND it MUST NOT imply that those modes have been successfully enforced unless later evidence proves enforcement.

### Requirement: Runtime diagnostics preserve quiet and machine-readable output [r[operator_diagnostics.quiet_machine_output]]

Mantle MUST keep default human output and default machine-readable output free of the verbose runtime fingerprint. When `--json` is selected, stdout MUST remain reserved for the command's documented JSON result unless the command explicitly documents a different machine-readable stream. If diagnostics are enabled with JSON output, Mantle MUST send the runtime fingerprint to stderr or another documented diagnostic channel.

#### Scenario: Default output stays quiet [r[operator_diagnostics.quiet_machine_output.scenario.default-quiet]]

- GIVEN an operator runs a Mantle command without verbose or talkative diagnostics
- WHEN Mantle renders normal human output
- THEN Mantle MUST NOT print the runtime fingerprint
- AND it MUST NOT add version banners or context blocks to default stderr.

#### Scenario: JSON stdout remains parseable [r[operator_diagnostics.quiet_machine_output.scenario.json-stdout]]

- GIVEN an operator runs a Mantle command with `--json`
- WHEN the command succeeds or fails with a documented JSON result
- THEN stdout MUST contain only the documented JSON payload for that command
- AND verbose runtime diagnostics, when requested, MUST be emitted outside stdout.

### Requirement: Verbose diagnostics are redacted [r[operator_diagnostics.redacted_runtime_diagnostics]]

Mantle MUST redact or omit secret-bearing runtime data from verbose diagnostics. Runtime fingerprints MUST NOT include signing-key secret material, bearer tickets, trusted private keys, authentication tokens, raw environment values, or full argv strings that can embed secrets. Secret-like configuration should be summarized by non-secret counts or public identifiers only.

#### Scenario: Secret-bearing fields are omitted [r[operator_diagnostics.redacted_runtime_diagnostics.scenario.omit-secrets]]

- GIVEN a Mantle command uses signing keys, trusted keys, bearer tickets, environment values, or user-provided arguments that may contain secrets
- WHEN Mantle emits a verbose runtime fingerprint
- THEN the fingerprint MUST omit secret bytes and raw secret-bearing strings
- AND it MAY include non-secret summaries such as counts, public verifier names, or selected mode labels.

#### Scenario: Diagnostic context stays bounded [r[operator_diagnostics.redacted_runtime_diagnostics.scenario.non-claim]]

- GIVEN Mantle emits a runtime fingerprint
- WHEN an evidence file, status reply, or log summary cites that fingerprint
- THEN the cited claim MUST be limited to the runtime context selected for the process
- AND it MUST NOT claim validation, build success, hermetic enforcement, substitution trust, or reproducibility from the fingerprint alone.

### Requirement: Project soundness diagnostics are stable and machine readable [r[operator_diagnostics.project_soundness_diagnostics]]

Mantle MUST render project soundness diagnostics with stable class identifiers, severity, subject, message, evidence references when available, and bounded fix guidance. JSON output MUST remain parseable and MUST include validity, issue count, highest severity, and ordered issues without mixing human diagnostics into stdout.

#### Scenario: JSON soundness output is deterministic [r[operator_diagnostics.project_soundness_diagnostics.scenario.json]]

- GIVEN equivalent project soundness issues are discovered in different traversal orders
- WHEN Mantle renders JSON diagnostics
- THEN Mantle MUST emit the same ordered issue list and summary fields
- AND stdout MUST contain only the documented JSON payload when `--json` is selected.

#### Scenario: Human diagnostics name failure class [r[operator_diagnostics.project_soundness_diagnostics.scenario.human]]

- GIVEN `mantle check` finds project soundness issues
- WHEN Mantle renders human output
- THEN each issue MUST include a stable class or concise class label, affected input/patch/file subject, and bounded fix guidance
- AND default output MUST remain free of verbose runtime fingerprints unless diagnostics are explicitly requested.

#### Scenario: Check does not overclaim readiness [r[operator_diagnostics.project_soundness_diagnostics.scenario.non-claim]]

- GIVEN project soundness diagnostics report no issues
- WHEN Mantle summarizes the result
- THEN the claim MUST be limited to manifest, lockfile, generated input, and configured project-state consistency
- AND it MUST NOT claim build success, source availability, trust satisfaction, or reproducibility unless separate evidence proves those facts.

### Requirement: Nickel export diagnostics are stable and machine readable [r[operator_diagnostics.nickel_export_diagnostics]]

Mantle MUST render Nickel export diagnostics with stable schema labels, status fields, format, output target, receipt or digest references, and deterministic failure classes. JSON output MUST remain parseable and MUST NOT mix human diagnostics into stdout.

#### Scenario: JSON export output is parseable [r[operator_diagnostics.nickel_export_diagnostics.scenario.json]]

- GIVEN an operator runs a Nickel export command with `--json`
- WHEN the command succeeds or fails with a documented JSON result
- THEN stdout MUST contain only the documented export JSON payload
- AND human diagnostics, verbose fingerprints, and evaluator logs MUST be emitted outside stdout or captured as bounded fields.

#### Scenario: Human diagnostics name export failure class [r[operator_diagnostics.nickel_export_diagnostics.scenario.human-failure]]

- GIVEN a Nickel export request fails validation or evaluation
- WHEN Mantle renders human diagnostics
- THEN the diagnostic MUST include a stable failure class, affected source or import subject when safe to reveal, and bounded fix guidance
- AND it MUST NOT claim build success, deployability, or correctness beyond the failed export attempt.

#### Scenario: Export success remains a bounded claim [r[operator_diagnostics.nickel_export_diagnostics.scenario.non-claim]]

- GIVEN a Nickel export command succeeds
- WHEN Mantle summarizes the result
- THEN the summary MAY claim that the declared Nickel export produced the recorded digest under the recorded evaluator descriptor
- AND it MUST NOT claim that generated config is deployable, semantically valid for a frontend, or sufficient for a build unless separate evidence proves those facts.

### Requirement: Offline build runbook is discoverable and claim-bounded

r[operator_diagnostics.offline_build_runbook] Mantle MUST document and diagnose the offline build workflow as an explicit, claim-bounded operator path. The runbook and diagnostics MUST cover source-bundle export, source-bundle import with pinning, source-bundle verify or offline preflight, `mantle build --offline-source-preflight --no-substitute`, evidence inspection, and common blocker remediation without presenting source readiness, route eligibility, or offline Cargo evidence as build success or stronger proof than the underlying evidence supports.

#### Scenario: operator can find the offline command sequence

GIVEN an operator reads the README, operator workflow guide, or command reference
WHEN they look for an offline build workflow
THEN the docs MUST show the ordered source-bundle export/import/pin/preflight/build/evidence command sequence
AND the docs MUST name the expected human or JSON evidence fields to inspect.

#### Scenario: offline blockers include safe next actions

GIVEN an offline build or preflight finds missing, stale, unpinned, unsupported, untrusted, network-required, or malformed source/evidence state
WHEN Mantle renders human or JSON diagnostics
THEN the diagnostics SHOULD include a bounded next action such as exporting/importing/pinning a source bundle, inspecting adapter metadata, or rerunning an explicit non-offline workflow
AND JSON output MUST remain parseable with stable fields.

#### Scenario: runbook does not overclaim

GIVEN source-bundle readiness, route-plan eligibility, or offline Cargo evidence is cited by docs, diagnostics, tasks, or status replies
WHEN the claim is made
THEN the claim MUST stay within the underlying evidence boundary
AND it MUST NOT claim build success, output trust, Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness without separate evidence.

#### Scenario: sensitive diagnostic data stays redacted

GIVEN offline diagnostics mention stores, source records, trusted keys, substituters, remote builders, or environment-derived configuration
WHEN Mantle renders runbook hints or JSON diagnostics
THEN it MUST omit bearer tokens, private key paths, raw environment values, and unbounded source lists
AND it MAY include bounded counts, stable blocker classes, and digest references.

### Requirement: Release verification success is rendered only after acceptance

r[mantle.operator_diagnostics.release_verification.terminal_verdict] Mantle MUST render the human release verification success marker only when the completed final decision is valid, and MUST render explicit rejection wording without any success marker when selected policy fails.

#### Scenario: Human rejection cannot look successful

r[mantle.operator_diagnostics.release_verification.fixtures.negative]
- GIVEN an invocation passes manifest integrity but fails a required policy check
- WHEN human output is rendered
- THEN the process MUST exit nonzero and identify the rejection
- AND stdout and stderr MUST NOT contain `release evidence verified` or equivalent success wording.

#### Scenario: Human acceptance is terminal

r[mantle.operator_diagnostics.release_verification.fixtures.positive]
- GIVEN the completed final decision is valid
- WHEN human output is rendered
- THEN the success marker MUST appear only after every selected check is represented
- AND no later policy error MAY reverse that rendered verdict.

### Requirement: JSON release verification carries final validity

r[mantle.operator_diagnostics.release_verification.json_contract] Mantle JSON release verification output MUST include versioned top-level `valid`, closed `disposition`, ordered `checks`, and ordered `diagnostics` fields for accepted and policy-rejected decisions, with stdout containing exactly one JSON value.

#### Scenario: JSON rejection is parseable and non-promoting

r[mantle.operator_diagnostics.release_verification.json_negative]
- GIVEN a selected release policy is unsatisfied after manifest loading succeeds
- WHEN `mantle --json release verify` renders the final decision
- THEN stdout MUST contain one parseable result with `valid` set to false and a rejection disposition
- AND the process MUST exit nonzero without emitting a separate success payload or human text on stdout.

### Requirement: Verification rendering consumes the final decision

r[mantle.operator_diagnostics.release_verification.render_boundary] Human and JSON renderers MUST consume the immutable completed verification decision and MUST NOT perform or defer policy validation while rendering.

#### Scenario: Rendering cannot change validity

r[mantle.operator_diagnostics.release_verification.render_boundary.test]
- GIVEN an immutable accepted or rejected decision
- WHEN either renderer formats it
- THEN rendering MUST preserve validity, disposition, check ordering, and diagnostics
- AND it MUST have no authority to run a missing gate or convert a rejection into success.

### Requirement: Remote execution emits a canonical bounded telemetry model [r[operator_diagnostics.remote_execution_telemetry]]

Mantle MUST define provider-neutral bounded telemetry events and metric descriptors for route decisions, queue admission, assignment, attempt transitions, retry and stale-fence rejection, execution result, transfer demand/progress/resume/cutoff/fallback, output admission, and publication when those events occur. Event semantics MUST be owned by Mantle rather than by a Prometheus, OTLP, database, cloud, or dashboard adapter.

#### Scenario: Remote lifecycle events share stable semantics

- GIVEN a remote job moves through planning, assignment, execution, transfer, admission, and optional publication
- WHEN Mantle emits telemetry
- THEN events MUST use stable schema, phase, result, route, retry, transfer, and reason classes with bounded attributes
- AND equivalent lifecycle facts MUST normalize to equivalent event meaning regardless of exporter choice.

#### Scenario: Metrics keep bounded cardinality

- GIVEN telemetry includes job, attempt, worker, output, path, trace, error, ticket, key, or provider facts
- WHEN Mantle maps events to metric labels
- THEN labels MUST be limited to declared low-cardinality classes
- AND raw ids, store paths, output names, trace ids, bearer material, key material, credentials, and arbitrary error text MUST NOT become metric label values.

#### Scenario: Telemetry remains a diagnostic non-claim

- GIVEN telemetry reports a successful phase, transfer, or admitted result
- WHEN an operator or evidence summary cites it
- THEN the claim MUST be limited to the recorded runtime event and bound identities
- AND telemetry alone MUST NOT prove compiler correctness, source reproducibility, release eligibility, CI success, or physical-target determinism.

### Requirement: Telemetry exporters are optional isolated shells [r[operator_diagnostics.telemetry_exporter_isolation]]

Mantle MAY provide Prometheus and OTLP adapters over the canonical telemetry model, but exporter configuration MUST be typed, bounded, provider-neutral at the core boundary, and disabled unless explicitly selected. Exporter delay, backpressure, rejection, or outage MUST NOT change scheduler safety, build result, transfer identity, or output-admission truth.

#### Scenario: Configured exporter receives bounded telemetry

- GIVEN an operator explicitly enables a supported exporter with valid typed policy
- WHEN Mantle emits canonical remote events
- THEN the shell adapter MAY expose or export bounded normalized telemetry
- AND exporter-specific endpoints, credentials, batching, and network I/O MUST remain outside pure decision kernels.

#### Scenario: Exporter outage preserves build truth

- GIVEN a Prometheus endpoint cannot bind or an OTLP collector is unavailable, slow, or rejects a batch
- WHEN a remote build otherwise succeeds or fails
- THEN Mantle MUST preserve the original build and admission result and report exporter degradation separately
- AND it MUST drop, retry, or backpressure telemetry only within named bounded policy.

#### Scenario: Required observability evidence fails narrowly

- GIVEN an explicit operator policy requires durable observability evidence for a separate claim
- WHEN immutable log persistence or exporter delivery lacks the required receipt
- THEN Mantle MUST block only that observability-evidence claim with a deterministic diagnostic
- AND it MUST NOT rewrite an already established build or output-admission fact.
