## Why

Network access during ordinary derivation builds lets undeclared inputs enter outputs after action identity has already been computed. Mantle should keep fetching as an explicit fixed-output boundary and deny network access for normal builds unless a narrow compatibility policy explicitly admits and audits an exception.

## What Changes

- Require ordinary derivation execution to run with network disabled by default.
- Keep `fetchurl`, `fetchTarball`, and `fetchGit` network behavior inside fixed-output fetcher actions with declared hashes and retry policy.
- Model any compatibility network allowance as a declared sandbox capability that is policy-checked and receipt-reported.
- Add negative fixtures proving undeclared build-time network attempts fail closed.

## Impact

- **Files**: sandbox/network policy planning, fetcher/build dispatch boundary, report/audit events, docs, and Cairn build-correctness spec delta.
- **Testing**: positive fixed-output fetcher fixture; negative normal-build network attempt; negative undeclared compatibility allowance; Cairn validation and gates.

## Out of Scope

- Eliminating network from fixed-output source acquisition.
- Claiming the remote server or transport is trusted beyond declared hash verification.
