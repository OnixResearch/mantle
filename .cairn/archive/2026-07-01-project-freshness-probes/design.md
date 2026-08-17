# Design: Project freshness probes

## Architecture

Freshness is split into a pure observation core and an imperative probe shell.

- Pure core: probe schema validation, observation normalization, old-vs-new comparison, stale/unchanged/error classification, templated value substitution planning, lock update planning, bounded diagnostics, and non-claim classification.
- Imperative shell: Git/HTTP/local filesystem/command execution, timeout handling, environment and cwd setup, network policy enforcement, output capture, and report rendering.

The core receives `FreshnessObservation` records, not shell commands or network handles. Each observation should include probe kind, input name, observed value, value digest, command/probe identity digest when relevant, status, bounded stderr or diagnostic summary, and network-required/offline-blocked markers.

## Probe model

Built-in probes should cover common deterministic needs: Git reference resolution, HTTP text retrieval, HTTP JSON field extraction, local file digest/mtime summaries, directory tree freshness summaries, and explicit command probes. Command probes should use an argv-based contract with an explicit working directory, empty or whitelisted environment, output byte limit, timeout, success exit code set, and UTF-8 or byte-output mode.

URL/reference templates may consume the validated freshness value. Template rendering must be pure and bounded: undefined variables, oversized output, and invalid destination syntax fail before any fetch or lock update.

## Refresh flow

`mantle list-stale` executes or consumes observations, compares them with lockfile freshness values, and reports stale/unchanged/failed inputs without mutating files. `mantle refresh` uses the same observation plan, then fetches and hashes only selected stale inputs before updating `mantle.lock` and generated inputs.

Offline mode must not run network probes. It should classify inputs as `network-required` or `probe-unavailable` rather than silently trusting stale lock values.

## Validation strategy

Pure positive tests should cover built-in probe normalization, command observation normalization, stale classification, template substitution, selected-input filtering, and unchanged lock planning. Pure negative tests should cover empty values, oversized values, invalid templates, command failure observations, network-required observations in offline mode, mismatched input names, and unsupported probe versions.

Shell tests should use local fixtures for Git, HTTP, local directory, and command probes. Negative shell tests should prove timeouts, non-zero exits, output limit violations, missing commands, and no-network mode fail with deterministic diagnostics.
