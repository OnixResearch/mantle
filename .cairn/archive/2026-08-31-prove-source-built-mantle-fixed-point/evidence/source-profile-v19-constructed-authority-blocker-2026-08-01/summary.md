# Source profile v19: constructed-authority blocker

## Result

Pueue task `7920` failed closed after `158s`.
The current native source manifest correctly names two empty virtual store-path records:

- the fresh StageX runtime handoff;
- the fresh StageX intermediate provider.

The profile command also loaded that manifest through `--include-bundle`.
The supplemental payload validator rejected the first empty store-path record because it is not a materialized fixed fetcher input.

## Root cause

The native manifest has two roles.
It binds the current evaluated graph, including its StageX store-path dependencies.
It also supplies acquisition records for profile materialization.
The profile reader had treated every manifest record as an acquisition payload.

The two virtual records must remain in the separately bound native manifest.
They must not become profile payload records because the same proof constructs and imports those paths before native offline preflight.

## Repair

The supplemental bundle reader now selects only fixed fetcher records.
The exact-union validator applies the same projection to the separately bound native and StageX manifests.
It still rejects missing, conflicting, duplicate, or extra fetch records.

Pueue task `7107` passed 34 focused source-built fixed-point tests.
The tests include a positive current-manifest projection and negative missing/extra union checks.
Pueue task `7110` passed the source-bundle test module, strict first-party Clippy, and focused diff checks.

## Non-claim

This repair proves only the authority split between acquisition records and proof-constructed store paths.
It does not prove profile v19 generation, native-provider construction, the Mantle fixed point, or release eligibility.
