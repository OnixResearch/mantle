# Nickel Standard Library Specification

## Purpose

Defines the **minimal** Nickel contracts and conversion helpers shipped with
mantle for describing derivations. The stdlib is the thinnest possible layer
between user-written Nickel and the Rust glue — it defines *what a derivation
is*, not how to conveniently build software.

Builder templates, build phases, mkDerivation patterns, stdenv equivalents,
and input set helpers are NOT part of this stdlib. They belong in separate
Nickel packages that import the mantle stdlib and layer convenience on top.
mantle's core ships the schema; the ecosystem ships the opinions.
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
imports the mantle stdlib.

#### Scenario: Core stdlib has no builder logic

- GIVEN the mantle stdlib files in `lib/`
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

- GIVEN `{ name = "foo", builder = "/bin/sh" } | mantle.Derivation`
- WHEN evaluated
- THEN succeeds with defaults applied

#### Scenario: Contract catches missing name

- GIVEN `{ builder = "/bin/sh" } | mantle.Derivation`
- WHEN evaluated
- THEN Nickel reports a contract violation for missing `name`

#### Scenario: Extra field rejected

- GIVEN `{ name = "foo", builder = "/bin/sh", bogus = true } | mantle.Derivation`
- WHEN evaluated
- THEN Nickel rejects the extra field

#### Scenario: Builder package uses its own contract

- GIVEN `{ pname = "foo", version = "1.0", ... } | builders.MkDerivationArgs`
- WHEN evaluated by the builder package
- THEN pname and version are accepted by the builder contract
- AND the builder produces a record that the Rust glue layer can
  deserialize as a CrunchDerivation

### Requirement: System enum

The stdlib MUST define a `System` enum contract covering the supported target
platform identifiers.

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

The stdlib MUST define `HashAlgo` and `HashMode` enum contracts for fixed
outputs and fetchers.

```nickel
let HashAlgo = [| 'md5, 'sha1, 'sha256, 'sha512 |] in
let HashMode = [| 'flat, 'recursive |] in
```

#### Scenario: Hash mode typo caught

- GIVEN `mode = 'recusrive`
- WHEN the HashMode contract runs
- THEN Nickel rejects it as a non-matching variant

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
let libfoo = { name = "libfoo", ... } | mantle.Derivation in
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

The stdlib MUST define a `Sandbox` enum contract for build execution backends.

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

#### Scenario: Unknown sandbox backend rejected

- GIVEN `sandbox = 'docker`
- WHEN the Sandbox contract runs
- THEN Nickel rejects the unsupported sandbox variant

### Requirement: FixedOutput contract

The stdlib MUST define a `FixedOutput` contract covering the declared content
hash, algorithm, and hashing mode for fixed-output derivations.

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

#### Scenario: Fixed output mode defaults to flat

- GIVEN a FixedOutput record with `hash` and `algo` only
- WHEN the contract runs
- THEN `mode` defaults to `'flat`

### Requirement: StorePath validator

The stdlib MUST define a `StorePath` validator for absolute `/nix/store/...`
paths.

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

The stdlib MUST define a `Name` validator for derivation names.

```nickel
let Name = std.contract.from_validator (fun value =>
  if std.is_string value
    && std.string.length value > 0
    && std.string.is_match "^[a-zA-Z0-9+._?=-][a-zA-Z0-9+._?=-]*$" value
  then 'Ok
  else 'Error { message = "invalid derivation name: %{value}" }
) in
```

#### Scenario: Invalid derivation name rejected

- GIVEN `name = "bad name with spaces"`
- WHEN the Name contract runs
- THEN Nickel rejects the invalid derivation name

### Requirement: Enum-to-string conversion helpers

The stdlib MUST provide conversion helpers that turn enum values into the
string forms consumed by JSON export and string interpolation.

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

#### Scenario: System enum converts to string

- GIVEN `'x86_64-linux`
- WHEN `system_to_string` runs
- THEN it returns `"x86_64-linux"`

### Requirement: Documentation via doc annotations

All contracts and fields MUST have `| doc` annotations for `nickel query`
discoverability.

#### Scenario: Query a field

- GIVEN the mantle stdlib loaded
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

Users import one file: `let mantle = import "lib.ncl" in`

#### Scenario: Entry point re-exports core contracts only

- GIVEN `lib/lib.ncl`
- WHEN it is inspected
- THEN it re-exports the core derivation contracts and helpers
- AND it does not re-export builder-layer helpers like `mkDerivation` or `mkShell`

### Requirement: Builder package location

A builder package MUST exist at `builders/lib.ncl` that provides
mkDerivation, mkStdenv, mkShell, callPackage, and MkDerivationArgs.
This package is separate from the core stdlib.

`builders/mk_derivation.ncl` imports `derivation.ncl` and
`contracts.ncl` from the stdlib via the Nickel import path (not
relative imports, to avoid circular imports with `builders/lib.ncl`).

`mkShell` MUST build a derivation that materializes shell activation
metadata as `$out/.crunch-shell.json`.

That sidecar MUST:
- carry a required `version` field set to `1`
- carry `env` entries derived from the shell author's declared shell env vars
- carry `path_entries` derived from the shell author's declared inputs
- carry an optional `hook` string, represented as null/absent when not set

The sidecar format exists so shell activation metadata is machine-readable and
independent from the core derivation contract.

#### Scenario: Builder package exists

- GIVEN the mantle source tree
- WHEN `builders/lib.ncl` is inspected
- THEN it contains mkStdenv, mkDerivation, mkShell, callPackage

#### Scenario: mkShell writes a sidecar with versioned metadata

- GIVEN a Nickel expression using `builders.mkShell` with env vars and
  build inputs
- WHEN that shell derivation is evaluated and built
- THEN its output contains `.crunch-shell.json`
- AND the sidecar JSON contains `version = 1`
- AND the sidecar `env` contains the declared shell env vars
- AND the sidecar `path_entries` contains the derived shell PATH entries

#### Scenario: mkShell without hook records null or absent hook

- GIVEN a Nickel expression using `builders.mkShell` with no hook field
- WHEN the shell derivation is built
- THEN `.crunch-shell.json` exists
- AND its `hook` field is null or absent

#### Scenario: Bootstrap uses raw Derivation contract

- GIVEN `bootstrap/hello.ncl`
- WHEN it needs to build something
- THEN it uses `mantle.Derivation` directly (bootstrap files do not
  use mkDerivation)

### Requirement: Stdlib ships with the binary

The `.ncl` files MUST be embedded in the mantle binary at compile time
(`include_str!`) and written to a known path at runtime, or resolved via
`--import-path`. The stdlib MUST also be usable directly from the source
tree during development.

#### Scenario: Binary can resolve stdlib without checkout-relative imports

- GIVEN a built mantle binary outside the source tree
- WHEN it evaluates a Nickel file importing `lib.ncl`
- THEN the stdlib resolves successfully

### Requirement: No Nix string context

The stdlib MUST NOT implement NixString-style context tracking. Inputs are
declared explicitly in the `inputs` field. String interpolation in Nickel
is plain string concatenation — no hidden metadata. The sandbox enforces
hermeticity: if a store path is referenced in a string but not in `inputs`,
the build fails.

#### Scenario: Undeclared interpolated store path is not implicitly tracked

- GIVEN a build script string that mentions a store path not listed in `inputs`
- WHEN the build runs
- THEN mantle does not infer that dependency from string context alone
- AND the build fails unless the input is declared explicitly

### Requirement: Extensibility by external packages

The stdlib MUST be designed so that external Nickel packages can build on
it via merge.

#### Scenario: External builder package can layer on Derivation

- GIVEN an external Nickel package that imports `lib.ncl`
- WHEN it merges its own builder helpers on top of `mantle.Derivation`
- THEN those helpers can produce valid derivation-shaped records without
  modifying the core stdlib

#### Scenario: Core stdlib stays minimal while external packages extend it

- GIVEN the mantle stdlib and a separate builder package
- WHEN both are inspected together
- THEN the stdlib defines the contracts and helpers
- AND the external package provides higher-level builders without editing the stdlib

Example shape:

```nickel
# Hypothetical external package: crunch-builders
let mantle = import "crunch/lib.ncl" in

{
  bash_builder = {
    script | String,
    _seed,
    builder = "%{_seed.bash}/bin/bash",
    args = ["-c", script],
    inputs | default = [_seed.bash, _seed.coreutils],
  } | mantle.Derivation & { .. },

  mk_derivation = { ... },
}
```

The core stdlib enables this pattern but does not ship these builders itself.

