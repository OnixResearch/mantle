# Design: Nickel export actions

## Architecture

The export path is split into a pure planning core and a thin evaluation shell.

- Pure core: validates the export request shape, allowed formats, import path policy, output-target policy, declared dependency refs, and evaluator descriptor shape. It plans the receipt fields without reading files or executing Nickel.
- Imperative shell: resolves file paths, reads source bytes, invokes the Nickel evaluator, writes the requested output when `--out` is supplied, computes BLAKE3 source/output digests, and renders diagnostics.

The same receipt model should be usable whether the export is invoked directly by `mantle export` or by a later project file-generation workflow.

## Export request

A normalized export request contains:

- root source files (`srcs`) and dependency files (`deps`)
- safe import paths relative to the declared project or invocation root
- requested format: JSON first, with TOML, YAML, and raw as follow-on formats
- output target policy: stdout-only or explicit output file
- evaluator descriptor: Nickel binary object ref or digest, Nickel version, and root evaluator options

Absolute import paths and paths that normalize above the root fail before evaluation. Missing declared deps, undeclared import use when provenance is required, evaluator mismatch, and output digest mismatch fail closed for strong correctness claims.

## Diagnostics

Human output should name the output target, format, receipt digest, and bounded failure class. JSON output should contain a stable schema label, success flag, format, source refs, evaluator descriptor summary, output digest, and diagnostics. Verbose runtime fingerprints remain outside default stdout and must not turn export success into broader build or deploy claims.

## Validation strategy

Positive tests should cover JSON export, declared dependency imports, safe import paths, stdout-only operation, output-file operation, and receipt determinism. Negative tests should cover absolute imports, `..` escapes, missing deps, unsupported formats, evaluator-version mismatch fixtures, and JSON stdout contamination.
