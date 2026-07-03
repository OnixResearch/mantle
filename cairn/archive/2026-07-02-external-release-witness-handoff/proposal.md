## Why

Mantle has provider-bound release evidence with a local/provider-bound witness replay and satisfied single-witness policy, but the current release note explicitly says not to describe it as an external independent rebuild unless a separate operator supplies the witness sidecar.

The next trust step is an operator-facing external witness handoff: export the request, have an independently operated witness replay it, import the returned sidecars, and record the final verification evidence without weakening the claim boundary.

## What Changes

- Produce a public-only witness request for the current provider-bound release evidence bundle.
- Define the expected external witness response shape and import checks.
- Record final release verification after importing the external witness sidecars.
- Update release notes/docs with bounded wording that distinguishes provider-bound local quorum from external independent witness agreement.

## Impact

- **Files**: release evidence notes, Cairn evidence, possibly operator workflow docs if a missing handoff instruction is found.
- **Generated artifacts**: request directory, witness scratch, imported witness sidecars, and final verification JSON under ignored or durable release-artifact storage.
- **Testing**: release-verify positive path with imported witness, negative import/verify cases, and Cairn validation/gates.
