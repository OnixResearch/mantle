## Why

Mantle can validate and render an existing Nix-free demo bundle summary, but operators still need to assemble the bundle by hand. A generator would make demos reproducible, reduce missing-file mistakes, and produce a self-contained artifact with the same validation rules as the existing CLI.

## What Changes

- Add a `mantle nix-free-demo generate` command or equivalent script that creates a complete demo bundle directory.
- Generate `summary.json`, README, command transcripts, receipt/digest references, and artifact digest files from explicit inputs.
- Reuse the existing summary validation and README rendering core.
- Keep non-claims explicit when the generator uses mocked or blocked proof evidence.

## Impact

- **Files**: CLI command/script, pure bundle manifest builder, tests, docs, and fixture bundle.
- **Testing**: positive bundle generation/validate/readme round trip and negative missing input/non-claim/malformed summary cases.

## Out of Scope

- Running the expensive fixed-point proof automatically inside the generator.
- Claiming Nix-free success from blocked or synthetic evidence.
- Uploading or publishing bundles.
