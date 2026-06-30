# Design: Project soundness diagnostics

## Architecture

Project soundness diagnostics should be computed as pure classifications over parsed project state, then rendered by the CLI.

- Pure core: manifest/lock/generated-input comparison, schema compatibility, input identity matching, patch reference checks, mirror validation results, freshness/trust/fetch/retention diagnostic normalization, severity assignment, exit classification, and JSON-ready data shaping.
- Imperative shell: file reads, optional probe/trust execution in explicit modes, generated-input file comparison, terminal coloring, JSON rendering, and process exits.

Diagnostics should use stable class identifiers so tests, agents, and docs can cite exact failure modes.

## Check modes

Default `mantle check` should be no-network and should validate static soundness: manifest parse, schema version, lock parse, lock/manifest kind matches, generated input freshness relative to lock, undefined patches, mirror syntax, hash algorithm support, fetch policy compatibility, trust policy shape, retention root state, and orphaned lock entries.

Explicit modes may run freshness probes or trust verification. Those modes must say what side effects or network access are possible and must keep JSON stdout parseable.

## Rendering

Human output should group diagnostics by input/patch and include concise fix guidance. JSON output should include version, validity, issue count, highest severity, class, subject, message, evidence refs when available, and non-claim notes. Project soundness may report project files internally consistent; it must not claim build success.

## Validation strategy

Pure positive tests should cover clean project state, warnings that do not invalidate soundness, JSON issue ordering, and generated-input match. Pure negative tests should cover each diagnostic class, duplicate/conflicting issues, stale generated inputs, kind mismatch, unsupported hash algorithm, fetch policy conflict, trust policy failure facts, retention root mismatch, and no-network classification.

CLI tests should prove default check does not execute network probes, JSON output is parseable, and explicit probe/trust modes label their behavior.
