## Why

Distributed build access and output trust must stay separate in every path. A ticket can authorize CPU/storage usage, but accepting a returned output requires signed metadata, requested identity checks, attestation policy, and key-material trust.

## What Changes

- Add a single remote-output admission pipeline used by remote execution, remote route planning, and remote report rendering.
- Preflight output trust before remote dispatch when builder signing keys or required attestation authorities are known.
- Verify returned outputs by store prefix, requested output identity, PathInfo signatures, object digests, artifact attestations, revocation state, and same-name-different-key checks.
- Report trust roots without exposing private key paths or bearer ticket secrets.

## Impact

- **Files**: remote output admission core, client preflight, import path, trust-policy loading, JSON/human reports, tests, and Cairn remote-builds spec delta.
- **Testing**: positive trusted-key import; negative no-output-trust preflight, unknown key, same-name-different-key, revoked/expired key, wrong store prefix, stale output identity, missing attestation, and tampered object fixtures.

## Out of Scope

- Adding new trust roots.
- Trusting the coordinator as an output authority.
- Claiming compiler correctness from a signed output alone.
