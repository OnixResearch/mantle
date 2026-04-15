## ADDED Requirements

### Requirement: Delta-aware substitution negotiation

The substitution pipeline MUST allow a trusted remote cache to advertise a
delta-capable transfer path in addition to ordinary full-artifact fetch.

For the first transport shape, a delta-capable HTTP cache MUST expose its delta
negotiation and streaming endpoints under the same cache authority used for
ordinary substitution.

When both sides support delta transfer, crunch SHOULD prefer the delta path if
receiver-local reuse can reduce transferred bytes. If capability negotiation
fails or reuse is not available, crunch MUST fall back to ordinary substitution
behavior.

#### Scenario: Delta-capable cache hit reuses local content

- GIVEN a trusted remote cache that supports delta transfer
- AND the receiver already has reusable blob chunks for the requested output
- WHEN crunch requests that output from the cache
- THEN crunch may fetch only the missing content instead of the whole artifact
- AND a successful result is reported as a normal substitution cache hit

#### Scenario: Delta-capable HTTP cache uses the existing authority

- GIVEN a trusted HTTP cache that supports both ordinary substitution and delta
  transfer
- WHEN crunch negotiates a delta-capable fetch from that cache
- THEN the delta negotiation and stream requests go to the same cache authority
  as the ordinary substitution request

#### Scenario: Legacy cache falls back to full-artifact fetch

- GIVEN a trusted remote cache that does not support delta transfer
- WHEN crunch requests that output from the cache
- THEN crunch uses the existing full-artifact substitution path
- AND the request does not fail merely because delta support is absent
