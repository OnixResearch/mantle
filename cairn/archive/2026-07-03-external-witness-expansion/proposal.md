## Why

One Aspen witness proves an externally replayed release for the current policy, but broader confidence requires multiple independent operators, keys, host classes, and source acquisition routes. Mantle needs a durable witness-expansion track that records how additional signed witnesses are requested, imported, evaluated, and counted without silently treating same-domain or unverifiable witnesses as independent agreement.

## What Changes

- Add an external witness roster/report workflow for release evidence that summarizes requested, returned, skipped, failed, and policy-counted witnesses.
- Extend release/global reproducibility evidence to preserve witness independence domains, host classes, source mode, signer key identity, digest match, and policy decision.
- Add fail-closed checks for duplicate identities, same-domain witnesses, unknown keys, bad signatures, wrong-release digests, and stale witness requests.
- Document the minimum quorum profiles needed before making stronger than single-witness claims.

## Impact

- **Files**: release verification/witness reporting, policy docs, tests, release notes, Cairn verification-evidence spec delta.
- **Testing**: positive multi-witness quorum, negative same-domain and unknown-key witnesses, wrong-release-digest rejection, Cairn validation/gates.
