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
