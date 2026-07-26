# Mantle artifact-auth Radicle source cutover

## Accepted source

- RID: `rad:z4JGYYW7WsesXUq7MXVdx16Fawu2f`
- HTTPS Git: `https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git`
- Reviewed commit: `799459346d5416fbd7b9f55840a7371441b55afa`
- Source archive BLAKE3: `246a7cad91e7e8a158e22da21f3bff3e61aa0431a58936b5a739178bc62064c7`
- Publication revision: `e41340bec587b6d049b5cc518ec7db925dde84be`
- Publication receipt BLAKE3: `e58a3de4d6b3b32a547c3cfe5c3e829292cda73891c7776f214f5d4edce10b1c`

## Lock agreement

Cargo regenerated `Cargo.lock` after both Mantle manifests moved from GitHub SSH to Radicle HTTPS. Nix regenerated only the `artifactAuthSource` lock node. Both package entries remain version `0.1.0` at the exact reviewed commit. The Nix `narHash` remains `sha256-nEgz2FtVuDesX95yyxidp0vhjxL4INB6Ve8rkpLyJk0=`.

Executable manifests and locks contain no `github.com/OnixResearch/artifact-auth` fallback. Historical lifecycle evidence remains unchanged.

## Behavioral evidence

Before the cutover at `ece3be0ab1ecfa0e0375b3f08ecceaaee37a643d`, focused artifact-auth tests passed for five pure action-result mapping cases and fourteen shell/operational receipt cases. The same focused positive and adversarial cases pass after the cutover. Focused no-dependency Clippy and workspace formatting pass. No Rust implementation file changed.

## Authority boundary

Mantle's legacy action-result decision remains authoritative, standalone authority remains unadmitted, and Mantle retains signing, trust/currentness collection, repository, registry, cache, build, receipt, and release decisions. This receipt does not prove source correctness, forge availability, whole-Mantle correctness, whole-stack GitHub independence, or release readiness.
