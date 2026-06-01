## Why

Mantle's example tree is useful but not yet treated as a supported surface. Examples mix beginner snippets, generated seed-dependent snippets, benchmark entry points, real-network fetchers, and heavyweight bootstrap demos without a machine-readable contract that says which examples must evaluate, build, execute, or stay manually run. That makes README drift, stale Crunch wording, and untested examples easy to introduce.

## What Changes

- Add a source-controlled examples support contract that catalogs each example, support tier, required tools, network behavior, and expected validation rail.
- Add deterministic drift checks so `examples/README.md` and the root README cannot omit supported examples or advertise non-existent paths.
- Keep generated or host-specific examples explicitly labeled so validation skips are narrow and evidence-backed instead of silent.
- Preserve Mantle naming in user-facing example prose while allowing exact `crunch` identifiers only where compatibility surfaces still require them.

## Impact

- **Files**: `examples/`, `examples/README.md`, root `README.md`, tests or tools that read the catalog, and possibly checked-in fixture metadata.
- **Testing**: catalog parser/checker tests, README drift tests, `cairn validate --root .`, and focused examples inventory tests.
