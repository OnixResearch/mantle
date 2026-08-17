# Design: Verbose runtime diagnostics

## Architecture

This is an imperative-shell feature with a pure formatting core. CLI argument parsing and command dispatch already know the global context: binary version, chosen command, physical output store, state directory, logical store prefix, JSON mode, verbose flag, explicit log level, and command-specific options such as hermeticity or substitution mode. The implementation should construct a small in-memory `RuntimeDiagnosticFingerprint` and pass it to a pure renderer. The shell decides whether to emit it.

The pure core should validate and render only redacted, stable fields. It should not read environment variables, inspect files, open logs, query the store, or print directly. The shell emits the rendered fingerprint on stderr or through the configured tracing layer before long-running or mutating work begins.

## Triggering rules

Emit the fingerprint when any of these are true:

- the operator passes `--verbose`;
- the operator selects an explicit log level that is at or above the project-defined talkative threshold;
- a command has an explicit diagnostic/preflight mode whose documented output includes runtime context.

Do not emit it for default command execution, default `--json` mode, or machine-readable subcommands unless verbosity is explicitly requested. If `--json` and verbosity are both requested, stdout remains reserved for the command's JSON result; diagnostics go to stderr or a structured diagnostic side channel.

## Fingerprint fields

Use stable field names and redaction. The initial field set should include:

- `mantle_version` from the compiled package version;
- `command` as a sanitized top-level command/subcommand label;
- `logical_store_prefix`;
- `physical_store_dir`;
- `state_dir`;
- `json_mode`;
- `verbosity_source`;
- `hermeticity_mode` when the command has one;
- `substitution_mode` and substituter count when the command has substitution behavior;
- `nix_compat_mode` when the logical prefix is selected through the compatibility switch.

Do not include signing-key secret material, trusted private key paths, bearer tickets, raw environment values, full argv, or user-provided strings that could contain secrets. Paths already selected as global store/state dirs may be printed because they are explicit runtime context, but future secret-bearing path fields should be redacted or omitted.

## Output shape

Human diagnostics can be a short multi-field line or a compact block. Structured diagnostics should use the same field names. The output must be easy to paste into evidence and support tickets without implying success. It should say only what was selected for this process, not what was verified.

## Validation strategy

- Pure tests for trigger decisions and redacted rendering.
- Positive CLI tests that `--verbose` build/store-style commands emit the fingerprint before work starts.
- Negative CLI tests that default human output and default `--json` output do not contain the fingerprint.
- JSON-mode tests proving stdout remains parseable when `--json --verbose` is used and diagnostics go to stderr.
- Redaction tests proving secret-like fields are omitted or summarized.
