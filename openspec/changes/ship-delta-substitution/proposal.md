# Ship delta substitution

## Why

crunch already has protocol-level delta transfer specs, but the ordinary
substitution path still behaves like full-artifact fetch in practice. That
means large remote cache hits transfer more bytes than necessary even when the
receiver already has reusable local content.

The next useful step is not another protocol document. It is shipping the delta
path inside the existing HTTP substitution flow with clear fallback behavior and
operator-visible reporting.

## What Changes

- integrate delta negotiation and streaming into the trusted HTTP substituter
  path
- build bounded receiver compatibility manifests from local `PathInfo` and
  castore presence
- reconstruct ordinary substitution acceptance from the completed delta result
- report whether a substitution used full fetch or delta reuse, including byte
  counts and fallback reasons

## Capabilities

### New Capabilities
- `delta-http-substitution`: trusted HTTP substituters can satisfy cache hits
  through delta transfer instead of whole-artifact fetch
- `receiver-compatibility-manifest`: crunch can describe what reusable content
  it already has for a requested output or closure
- `delta-reuse-reporting`: human and machine-readable reports show when delta
  reuse happened and how much data it saved

## Impact

- **Files**: substitution path in `crunch-store` or related cache code,
  reporting surfaces, HTTP test fixtures, docs for cache operators
- **APIs**: new internal delta negotiation and manifest-building helpers
- **Dependencies**: none required if existing HTTP stack is reused
- **Testing**: same-authority negotiation, bounded manifest building,
  fallback-to-full transfer, signed final acceptance, and reporting coverage

## Non-Goals

- peer-to-peer delta transfer in this change
- non-HTTP substituter transports
- changing trust policy for accepted cache hits
- inventing a second acceptance path separate from ordinary substitution
