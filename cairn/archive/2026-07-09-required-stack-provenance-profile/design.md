## Context

Mantle intentionally treats Valence stack provenance as opaque external evidence. The current CLI can require stack provenance at verification time, but release policy is operator-selected rather than profile-declared. Repeated stack releases should not rely on remembering a flag.

## Design

### Release profile contract

Add a profile entry for stack releases. The profile declares:

- stack-provenance mode: `required`;
- required sidecar role and schema;
- required Valence graph-report role and schema;
- claim scope: `identity-linkage-sidecar`;
- required non-claim boundary;
- release binary selection rule when multiple binaries are bundled.

### Constant generation

Move role/schema/scope/non-claim literals into one reviewed contract or generated constants source consumed by release verification. This avoids divergence between CLI text, Rust manifest constants, README examples, and Valence contracts.

### Verification behavior

Generic `mantle release verify` remains optional unless the selected profile requires stack provenance. When the profile requires it, verification must fail closed for missing sidecar, missing graph report, digest mismatch, wrong role/schema, wrong binary link, unsupported claim scope, or weakened non-claims.

## Alternatives

### Make stack provenance required globally

Rejected. Mantle is a general build/release tool and should not require Valence for non-stack releases.

### Keep relying on manual `--stack-provenance required`

Rejected for stack release classes. Profiles should encode release policy so CI and operators do not depend on remembered flags.

## Risks

- **Generic user surprise**: keep the default optional and require explicit profile selection.
- **String drift**: generate or check constants from one reviewed contract.
- **Semantic overreach**: keep Mantle's non-claim that Valence owns stack semantics and Mantle only validates bundle-local evidence linkage.
