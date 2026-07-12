# Source-root capabilities

Mantle reports source-root support per operation instead of advertising one
ambiguous capability:

```bash
mantle bootstrap capabilities
mantle --json bootstrap capabilities
```

The report is derived by a pure planner from shell-probed observations. It names
an executable command only when the operation has an implementation and its
required host tools were observed.

## Supported operation

`mantle bootstrap --source-root <manifest.json>` materializes the declared
source artifacts into the normalized seed contract. This operation is classified
as `host-assisted-source-materialization`, not as a full-source bootstrap. Its
output metadata records `full_source_bootstrap_eligible: false` and names these
materialization influences:

- the host C compiler and linker;
- host `make` and archive extraction tools; and
- host kernel and runtime libraries.

The source-root manifest and resulting output still have deterministic BLAKE3
identities. Those identities prove which declared inputs and output bytes were
observed; they do not erase host influence or establish a stage-zero bootstrap.

## Unsupported operation

`mantle self-build --source-root` is not an executable capability and is not
accepted by the CLI. The capability report lists `self-build-source-root` as
`unsupported` with no command because Mantle does not yet have a full-source
provider chain that can drive self-build without the legacy/bootstrap host
boundary.

Operators should use the supported bootstrap materialization command only for
its bounded host-assisted claim. StageX or another auditable seed-to-binary
lineage is required for stronger bootstrap claims.
