# Nickel Standard Library Specification

## Purpose

Defines the **minimal** Nickel contracts and conversion helpers shipped with
crunch for describing derivations. The stdlib is the thinnest possible layer
between user-written Nickel and the Rust glue — it defines *what a derivation
is*, not how to conveniently build software.

Builder templates, build phases, mkDerivation patterns, stdenv equivalents,
and input set helpers are NOT part of this stdlib. They belong in separate
Nickel packages that import the crunch stdlib and layer convenience on top.
crunch's core ships the schema; the ecosystem ships the opinions.

## Requirements

### Requirement: Scope boundary

The stdlib MUST contain only:

1. **Contracts** — Derivation, FixedOutput, Input, OutputRef
2. **Enums** — System, HashAlgo, HashMode, Sandbox
3. **Validators** — StorePath, Name
4. **Conversion helpers** — enum→string via match
5. **Fetchers** — fetchurl, fetchTarball, fetchGit (these produce
   Derivation records, not builder logic)
6. **Output selection** — the `select` function

The stdlib MUST NOT contain:

- `mkDerivation` or any phase-based builder
- `mkStdenv` or any stdenv-like wrapper
- `mkShell` or any dev environment helper
- `callPackage` or any dependency injection helper
- `MkDerivationArgs` or any builder-specific contract
- `overrideAttrs` or any override mechanism
- Default phase implementations (unpackPhase, configurePhase, etc.)

These MUST live in a separate Nickel package (`builders/`) that
imports the crunch stdlib.

#### Scenario: Core stdlib has no builder logic

- GIVEN the crunch stdlib files in `lib/`
- WHEN inspected
- THEN no file contains `mkDerivation`, `mkStdenv`, `mkShell`,
  phase names, or shell script templates

#### Scenario: Builder package imports stdlib

- GIVEN the builder package in `builders/`
- WHEN it defines `mkDerivation`
- THEN it does `import "derivation.ncl"` and `import "contracts.ncl"`
  via the import path, applying the Derivation contract to its output records

### Requirement: Derivation contract with enums and validators

The stdlib MUST provide a closed `Derivation` contract (no `..` tail).
Extra fields MUST be rejected. Builder-specific fields (`pname`,
`version`, `meta`, `passthru`, `overrideAttrs`) belong in the builder
package's contract, not the core Derivation contract.

The Derivation contract MUST allow exactly these fields:

- `name`, `builder`, `system`, `args`, `outputs`, `env`, `inputs`
- `fixed_output` (optional)
- `addressing_mode`
- `sandbox`

```nickel
{
  name | Name
    | doc "Derivation name. Used in the store path.",
  builder | String
    | doc "Absolute path to the build executable.",
  system | System
    | doc "Target platform."
    | default = 'x86_64-linux,
  args | Array String
    | doc "Arguments passed to the builder."
    | default = [],
  outputs | Array String
    | doc "Output names. Each gets a store path."
    | default = ["out"],
  env | { _ : String }
    | doc "Environment variables set during the build."
    | default = {},
  inputs | Array Input
    | doc "Build inputs. Each is either a Derivation record (built first) or a StorePath string (pre-existing)."
    | default = [],
  fixed_output | FixedOutput
    | doc "Fixed-output derivation parameters. Present only for FODs."
    | optional,
  addressing_mode | [| 'input-addressed, 'content-addressed |]
    | doc "How output paths are computed."
    | default = 'content-addressed,
  sandbox | Sandbox
    | doc "Sandbox backend. Default is native (bwrap on Linux)."
    | default = 'native,
}
```

#### Scenario: Valid derivation

- GIVEN `{ name = "foo", builder = "/bin/sh" } | crunch.Derivation`
- WHEN evaluated
- THEN succeeds with defaults applied

#### Scenario: Contract catches missing name

- GIVEN `{ builder = "/bin/sh" } | crunch.Derivation`
- WHEN evaluated
- THEN Nickel reports a contract violation for missing `name`

#### Scenario: Extra field rejected

- GIVEN `{ name = "foo", builder = "/bin/sh", bogus = true } | crunch.Derivation`
- WHEN evaluated
- THEN Nickel rejects the extra field

#### Scenario: Builder package uses its own contract

- GIVEN `{ pname = "foo", version = "1.0", ... } | builders.MkDerivationArgs`
- WHEN evaluated by the builder package
- THEN pname and version are accepted by the builder contract
- AND the builder produces a record that the Rust glue layer can
  deserialize as a CrunchDerivation

### Requirement: System enum

```nickel
let System = [|
  'x86_64-linux,
  'aarch64-linux,
  'x86_64-darwin,
  'aarch64-darwin,
|] in
```

#### Scenario: Typo caught

- GIVEN `system = 'x86_64_linux`
- WHEN the System contract runs
- THEN Nickel rejects it as a non-matching variant

### Requirement: Hash enums

```nickel
let HashAlgo = [| 'md5, 'sha1, 'sha256, 'sha512 |] in
let HashMode = [| 'flat, 'recursive |] in
```

### Requirement: Input contract

The stdlib MUST provide an `Input` contract that accepts either a
`Derivation` record or a `StorePath` string:

```nickel
let Input = std.contract.from_validator (fun value =>
  if std.is_string value then
    std.contract.apply StorePath 'dummy value |> match {
      _ => 'Ok
    }
  else if std.is_record value then
    'Ok  # Derivation contract applied when the full record is checked
  else
    'Error { message = "input must be a Derivation record or a StorePath string" }
) in
```

This allows mixed inputs naturally:

```nickel
let seed = import "seed.ncl" in
let libfoo = { name = "libfoo", ... } | crunch.Derivation in
{
  inputs = [
    seed.bash,       # string → source input
    seed.coreutils,  # string → source input
    libfoo,          # record → derivation input
  ],
}
```

On the Rust side, `#[serde(untagged)]` on an `Input` enum dispatches
automatically: string → `Input::Source`, object → `Input::Derivation`.

#### Scenario: Mixed inputs

- GIVEN `inputs = ["/nix/store/...-bash", { name = "lib", ... }]`
- WHEN the Input contract runs on each element
- THEN both pass (string validated as StorePath, record validated as Derivation)

#### Scenario: Invalid input

- GIVEN `inputs = [42]`
- WHEN the Input contract runs
- THEN a contract violation is reported

### Requirement: Sandbox enum

```nickel
let Sandbox = [| 'wasm, 'native, 'oci |] in
```

`'native` is the default. It uses the platform-specific sandbox
(bwrap on Linux). `'oci` uses an OCI container runtime. `'wasm`
uses a WASI runtime (wasmtime) but is limited to single-process
builds because standard WASI has no process spawning (`fork`,
`exec`, `spawn`). Build scripts that invoke multiple tools (bash
calling gcc, make, cp) cannot run under `'wasm`. See the
portability spec for the full rationale.

Once WASI gains subprocess support (on the standards roadmap),
`'wasm` may become the default.

### Requirement: FixedOutput contract

```nickel
let FixedOutput = {
  hash | String
    | doc "Expected hash in SRI or hex format.",
  algo | HashAlgo
    | doc "Hash algorithm.",
  mode | HashMode
    | doc "Hashing mode: flat (single file) or recursive (NAR)."
    | default = 'flat,
} in
```

### Requirement: StorePath validator

```nickel
let StorePath = std.contract.from_validator (fun value =>
  if std.is_string value
    && std.string.is_match "^/nix/store/[a-z0-9]{32}-.+$" value
  then 'Ok
  else 'Error { message = "expected a valid /nix/store/ path, got: %{value}" }
) in
```

#### Scenario: Invalid store path

- GIVEN a string `"/tmp/not-a-store-path"`
- WHEN the StorePath contract runs
- THEN a descriptive error is produced

### Requirement: Name validator

```nickel
let Name = std.contract.from_validator (fun value =>
  if std.is_string value
    && std.string.length value > 0
    && std.string.is_match "^[a-zA-Z0-9+._?=-][a-zA-Z0-9+._?=-]*$" value
  then 'Ok
  else 'Error { message = "invalid derivation name: %{value}" }
) in
```

### Requirement: Enum-to-string conversion helpers

```nickel
let system_to_string = fun sys => sys |> match {
  'x86_64-linux => "x86_64-linux",
  'aarch64-linux => "aarch64-linux",
  'x86_64-darwin => "x86_64-darwin",
  'aarch64-darwin => "aarch64-darwin",
} in

let hash_algo_to_string = fun algo => algo |> match {
  'md5 => "md5",
  'sha1 => "sha1",
  'sha256 => "sha256",
  'sha512 => "sha512",
} in

let hash_mode_to_string = fun mode => mode |> match {
  'flat => "flat",
  'recursive => "recursive",
} in
```

These are needed because the glue layer consumes JSON (where enums become
strings via Nickel's export), but they're also useful for Nickel code that
needs to interpolate system strings.

### Requirement: Documentation via doc annotations

All contracts and fields MUST have `| doc` annotations for `nickel query`
discoverability.

#### Scenario: Query a field

- GIVEN the crunch stdlib loaded
- WHEN `nickel query --field name crunch/lib/derivation.ncl`
- THEN the doc string, contract type, and default (if any) are displayed

### Requirement: Stdlib entry point

`lib/lib.ncl` MUST re-export:

```nickel
{
  Derivation, FixedOutput,
  StorePath, Name,
  System, HashAlgo, HashMode, Sandbox, Input,
  system_to_string, hash_algo_to_string, hash_mode_to_string,
  fetchurl, fetchTarball, fetchGit,
  Url, Hash,
  select,
}
```

It MUST NOT re-export mkStdenv, mkShell, mkDerivation, callPackage,
or MkDerivationArgs.

Users import one file: `let crunch = import "lib.ncl" in`

### Requirement: Builder package location

A builder package MUST exist at `builders/lib.ncl` that provides
mkDerivation, mkStdenv, mkShell, callPackage, and MkDerivationArgs.
This package is separate from the core stdlib.

`builders/mk_derivation.ncl` imports `derivation.ncl` and
`contracts.ncl` from the stdlib via the Nickel import path (not
relative imports, to avoid circular imports with `builders/lib.ncl`).

#### Scenario: Builder package exists

- GIVEN the crunch source tree
- WHEN `builders/lib.ncl` is inspected
- THEN it contains mkStdenv, mkDerivation, mkShell, callPackage

#### Scenario: Bootstrap uses raw Derivation contract

- GIVEN `bootstrap/hello.ncl`
- WHEN it needs to build something
- THEN it uses `crunch.Derivation` directly (bootstrap files do not
  use mkDerivation)

### Requirement: Stdlib ships with the binary

The `.ncl` files MUST be embedded in the crunch binary at compile time
(`include_str!`) and written to a known path at runtime, or resolved via
`--import-path`. The stdlib MUST also be usable directly from the source
tree during development.

### Requirement: No Nix string context

The stdlib MUST NOT implement NixString-style context tracking. Inputs are
declared explicitly in the `inputs` field. String interpolation in Nickel
is plain string concatenation — no hidden metadata. The sandbox enforces
hermeticity: if a store path is referenced in a string but not in `inputs`,
the build fails.

### Requirement: Extensibility by external packages

The stdlib MUST be designed so that external Nickel packages can build on
it via merge:

```nickel
# Hypothetical external package: crunch-builders
let crunch = import "crunch/lib.ncl" in

{
  bash_builder = {
    script | String,
    _seed,
    builder = "%{_seed.bash}/bin/bash",
    args = ["-c", script],
    inputs | default = [_seed.bash, _seed.coreutils],
  } | crunch.Derivation & { .. },

  mk_derivation = { ... },
}
```

The core stdlib enables this pattern but does not ship these builders itself.
