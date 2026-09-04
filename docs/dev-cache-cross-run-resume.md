# Development cache cross-run resume

Mantle can restore source-built fixed-point stage state into a new staging directory. This path is for development only.

## Enable resume

Use both options on a source-built fixed-point development run:

```text
--dev-provider-cache <absolute-cache-directory>
--dev-resume
```

`--dev-resume` without `--dev-provider-cache` fails before execution.

The first run executes the cold stage graph. It publishes a content-addressed manifest for each completed stage. Large provider payloads use one shared checkpoint reference.

A later run creates a new staging directory. It remeasures cache content before the pure resume core selects a stage prefix.

## Validation

A candidate must match all these facts:

- source-authority BLAKE3;
- plan BLAKE3;
- aggregate policy BLAKE3;
- stage kind and identifier;
- producer executable BLAKE3;
- output BLAKE3;
- execution-evidence BLAKE3;
- payload identities and bounds;
- bundle BLAKE3.

A missing, stale, modified, partial, conflicting, or unknown candidate does not authorize a skip. Mantle executes from an earlier admitted boundary.

## Report

Each dev attempt writes `dev-resume-report.json` in its staging directory. The report lists:

- stages restored from validated content;
- stages executed in the current attempt;
- the first incomplete stage;
- rejected cache identities and reasons;
- the provider-cache adoption disposition;
- explicit false values for promoted receipt and release-alias writes.

The machine schema is `schemas/machine-contracts/dev-resume-report.schema.json`.

## Promoted proof boundary

A promoted run does not read the dev resume namespace. Promoted checkpoint options reject dev cache, resume, and fast-fail options.

A restored stage was not executed again in the resumed attempt. Dev resume does not prove compiler correctness, promoted fixed-point success, release eligibility, or broad reproducibility.
