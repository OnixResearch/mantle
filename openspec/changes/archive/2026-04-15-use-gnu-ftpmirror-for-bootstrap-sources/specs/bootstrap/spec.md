## ADDED Requirements

### Requirement: GNU bootstrap sources use mirror front door

GNU-hosted bootstrap source URLs MUST use `https://ftpmirror.gnu.org/...`
instead of direct `https://ftp.gnu.org/gnu/...` endpoints.

The pinned source names and content hashes MUST remain unchanged when this
front-door URL changes, so the mirror policy improves reachability without
changing the intended source artifacts.

#### Scenario: Contributor inspects bootstrap source URLs

- GIVEN a contributor reads the checked-in bootstrap derivations
- WHEN they inspect GNU-hosted source URLs
- THEN they see `https://ftpmirror.gnu.org/...` URLs
- AND they do not see direct `https://ftp.gnu.org/gnu/...` URLs for those
  bootstrap tarballs

#### Scenario: Mirror front door preserves pinned artifact identity

- GIVEN a GNU-hosted bootstrap tarball already has a checked-in source name and
  content hash
- WHEN its URL is switched to `https://ftpmirror.gnu.org/...`
- THEN the source name stays the same
- AND the checked-in content hash stays the same
- AND the bootstrap flow still resolves the intended artifact through the
  mirror front door
