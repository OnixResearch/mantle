## ADDED Requirements

### Requirement: Flake-compat generation command

The CLI MUST provide a `crunch flake-compat` command that generates a Nix flake
shim for a crunch project.

Required behavior:
- By default, the command writes `flake.nix` at the discovered project root.
- Project-root discovery walks upward from the current working directory until
  it finds `crunch.ncl`.
- `--stdout` prints the generated flake text instead of writing a file.
- `--stdout` conflicts with `--output` and `--force`.
- `--output <path>` writes the generated file to an alternate path.
- Relative `--output` paths are resolved from the discovered project root.
- Missing parent directories for the selected output path are created
  automatically.
- the generated shim embeds `github:brittonr/crunch` as the default upstream
  crunch flake reference
- `--crunch-input <flake-ref>` overrides that embedded crunch flake reference
- `--force` overwrites an existing target file.
- Without `--force`, the command fails clearly if the target file already
  exists.
- The command fails clearly when run outside a directory tree containing
  `crunch.ncl`.
- The command never creates or mutates `flake.lock`.

The generated file content MUST be deterministic for the same logical project
inputs and content-shaping flags. Delivery mode (`--stdout` vs write target)
MUST NOT change the rendered bytes, and absolute checkout paths from the
generation machine MUST NOT affect the rendered output.

#### Scenario: Default write updates project flake

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat` runs there
- THEN `flake.nix` is written at that project root
- AND the file contains the generated crunch wrapper shim
- AND `flake.lock` is not created or modified

#### Scenario: Stdout mode is side-effect free

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat --stdout` runs
- THEN the generated `flake.nix` text is printed to stdout
- AND no file on disk is modified
- AND `flake.lock` is not created or modified

#### Scenario: Stdout conflicts with write-only flags

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat --stdout --output generated/flake.nix` runs
- THEN the command exits non-zero before generation
- AND the error explains that `--stdout` conflicts with `--output`

#### Scenario: Stdout conflicts with force

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat --stdout --force` runs
- THEN the command exits non-zero before generation
- AND the error explains that `--stdout` conflicts with `--force`

#### Scenario: Alternate output path works

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat --output generated/flake.nix` runs
- THEN the shim is written to `<project-root>/generated/flake.nix`
- AND missing parent directories for that target are created automatically
- AND the project-root `flake.nix` is left unchanged unless explicitly targeted
- AND `flake.lock` is not created or modified

#### Scenario: Default crunch input is rendered when not overridden

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat --stdout` runs without `--crunch-input`
- THEN the generated text embeds `github:brittonr/crunch` as the crunch flake
  input reference

#### Scenario: Custom crunch input changes only the embedded reference

- GIVEN a crunch project root with `crunch.ncl`
- WHEN `crunch flake-compat --stdout` is compared with
  `crunch flake-compat --stdout --crunch-input github:example/crunch`
- THEN both outputs are identical except for the embedded crunch flake
  reference field

#### Scenario: Existing flake requires explicit overwrite

- GIVEN a crunch project root whose target `flake.nix` already exists
- WHEN `crunch flake-compat` runs without `--force`
- THEN the command exits non-zero
- AND the error explains that the target already exists
- AND the existing file is left unchanged

#### Scenario: Force overwrite replaces existing target

- GIVEN a crunch project root whose target `flake.nix` already exists
- WHEN `crunch flake-compat --force` runs
- THEN the target file is replaced with the generated shim
- AND `flake.lock` is not created or modified

#### Scenario: Nested subdirectory still targets project root

- GIVEN a crunch project rooted at `<project-root>/crunch.ncl`
- AND the current working directory is `<project-root>/sub/dir`
- WHEN `crunch flake-compat` runs there
- THEN the project root is discovered at `<project-root>`
- AND the default target file is `<project-root>/flake.nix`

#### Scenario: Nested subdirectory keeps relative output anchored to project root

- GIVEN a crunch project rooted at `<project-root>/crunch.ncl`
- AND the current working directory is `<project-root>/sub/dir`
- WHEN `crunch flake-compat --output generated/flake.nix` runs there
- THEN the target file is `<project-root>/generated/flake.nix`
- AND the rendered bytes match generation from the project root with the same
  logical inputs

#### Scenario: Missing project root fails clearly

- GIVEN a directory tree without `crunch.ncl`
- WHEN `crunch flake-compat` runs there
- THEN the command exits non-zero
- AND the error explains that no crunch project root was found
