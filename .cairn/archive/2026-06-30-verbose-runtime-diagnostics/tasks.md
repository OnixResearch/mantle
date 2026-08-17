# Tasks

## Contract

- [x] [serial] Define the runtime fingerprint trigger policy, stable field names, command-specific mode fields, and non-claim wording. r[operator_diagnostics.verbose_runtime_fingerprint]
- [x] [serial] Define quiet/default behavior, JSON stdout preservation, and stderr or diagnostic-channel behavior under `--json --verbose`. r[operator_diagnostics.quiet_machine_output]
- [x] [serial] Define redaction rules for signing keys, bearer tickets, environment values, raw argv, trusted-key material, and other secret-bearing inputs. r[operator_diagnostics.redacted_runtime_diagnostics]

## Implementation

- [x] [serial] Implement a pure runtime-fingerprint model and renderer with trigger decisions and redaction summaries over in-memory command context. r[operator_diagnostics.verbose_runtime_fingerprint] r[operator_diagnostics.redacted_runtime_diagnostics]
- [x] [serial] Wire the CLI shell to construct and emit the fingerprint only when verbosity or diagnostic mode requires it, before long-running or mutating work starts. r[operator_diagnostics.verbose_runtime_fingerprint]
- [x] [serial] Preserve default human output and default `--json` stdout, routing diagnostics to stderr or a documented diagnostic channel when verbosity is enabled. r[operator_diagnostics.quiet_machine_output]

## Verification

- [x] [serial] Add pure positive tests for trigger decisions and field rendering when verbose diagnostics are selected. r[operator_diagnostics.verbose_runtime_fingerprint]
- [x] [serial] Add pure negative tests proving default mode does not trigger the fingerprint and secret-bearing fields are omitted or summarized. r[operator_diagnostics.quiet_machine_output] r[operator_diagnostics.redacted_runtime_diagnostics]
- [x] [serial] Add CLI positive tests proving representative `--verbose` build/store commands emit version, command label, logical store prefix, physical store directory, and state directory before work starts. r[operator_diagnostics.verbose_runtime_fingerprint]
- [x] [serial] Add CLI negative tests proving default human output and default `--json` stdout do not contain the fingerprint, and `--json --verbose` stdout remains parseable. r[operator_diagnostics.quiet_machine_output]
- [x] [serial] Add redaction tests proving signing key material, bearer tickets, raw env values, and raw argv strings are absent from diagnostics. r[operator_diagnostics.redacted_runtime_diagnostics]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation claims, then record focused implementation evidence before checking tasks complete. r[operator_diagnostics.verbose_runtime_fingerprint]
