# Derivation Glue Specification

## Purpose

Defines how evaluated Nickel values are converted to `nix_compat::Derivation`
structs suitable for building. The glue layer receives `NickelValue` from
crunch-eval and deserializes it directly into typed Rust structs via serde
(`NickelValue` implements `serde::Deserializer`). No JSON intermediate.

## Requirements

### Requirement: Typed Rust structs via serde Deserialize

The glue layer MUST define Rust structs with `#[derive(serde::Deserialize)]`
that mirror the Nickel Derivation contract:

```rust
#[derive(Deserialize)]
pub struct CrunchDerivation {
    pub name: String,
    pub builder: String,
    pub system: System,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_outputs")]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub inputs: Vec<Input>,
    pub fixed_output: Option<FixedOutput>,
}

/// An input is either a derivation to be built or a pre-existing store path.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Input {
    /// A derivation that must be built first. Its outputs become available
    /// in the sandbox. Added to `Derivation.input_derivations`.
    Derivation(Box<CrunchDerivation>),
    /// A pre-existing store path (e.g., from the seed toolchain).
    /// Must already exist in the store. Added to `Derivation.input_sources`.
    Source(StorePath),
}

#[derive(Deserialize)]
pub enum System {
    #[serde(rename = "x86_64-linux")]
    X86_64Linux,
    #[serde(rename = "aarch64-linux")]
    Aarch64Linux,
    // ...
}

#[derive(Deserialize)]
pub struct FixedOutput {
    pub hash: String,
    pub algo: HashAlgo,
    #[serde(default)]
    pub mode: HashMode,
}
```

Deserialization from `NickelValue` handles the type mapping automatically:
Nickel records → Rust structs, Nickel arrays → `Vec`, Nickel enum tags →
Rust enums, Nickel `null`/absent → `Option::None`.

Unknown fields (including `_`-prefixed fields that survived export)
SHOULD be handled via `#[serde(deny_unknown_fields)]` or ignored via
`#[serde(flatten)]` depending on extensibility requirements.

#### Scenario: Minimal derivation

- GIVEN a `NickelValue` representing `{ name = "hello", builder = "/bin/sh", system = 'x86_64-linux }`
- WHEN deserialized into `CrunchDerivation` and converted to a `nix_compat::Derivation`
- THEN `system` is `System::X86_64Linux`, outputs default to `["out"]`,
  environment contains `"out"` mapped to the computed output path

#### Scenario: With environment variables

- GIVEN `env` containing `{ CC = "/nix/store/.../bin/gcc", CFLAGS = "-O2" }`
- WHEN deserialized and converted
- THEN both key-value pairs appear in `Derivation.environment`, alongside
  auto-populated entries

### Requirement: Two kinds of inputs

The `inputs` array MUST support two kinds of entries:

1. **Derivation inputs** — full derivation records that must be built first.
   Added to `nix_compat::Derivation.input_derivations`.
2. **Source inputs** — pre-existing store paths (e.g., seed toolchain paths).
   Added to `nix_compat::Derivation.input_sources`. Must already exist in
   the store; crunch does not build them.

In Nickel, this is expressed naturally:

```nickel
let seed = import "seed.ncl" in
let libfoo = { name = "libfoo", builder = "...", ... } | crunch.Derivation in
{
  name = "myapp",
  builder = "%{seed.bash}/bin/bash",
  inputs = [
    seed.bash,       # string → Source input
    seed.coreutils,  # string → Source input
    seed.gcc,        # string → Source input
    libfoo,          # record → Derivation input
  ],
  ...
}
```

On the Rust side, `#[serde(untagged)]` on the `Input` enum lets serde
automatically dispatch: if the JSON value is a string, it's a `Source`;
if it's an object, it's a `Derivation`.

#### Scenario: Seed paths as source inputs

- GIVEN `inputs = ["/nix/store/...-bash-5.2", "/nix/store/...-gcc-13.2"]`
- WHEN processed by the glue layer
- THEN both appear in `Derivation.input_sources` and are mounted in the
  sandbox

#### Scenario: Mix of sources and derivations

- GIVEN `inputs = [seed.bash, libfoo]` where `seed.bash` is a string and
  `libfoo` is a derivation record
- WHEN processed
- THEN `seed.bash` goes to `input_sources`, `libfoo` is recursively
  converted and goes to `input_derivations`

### Requirement: Derivation input resolution

For each `Input::Derivation` in the `inputs` array, the glue layer MUST:

1. Recursively convert the nested `CrunchDerivation` to a `nix_compat::Derivation`
2. Compute its derivation path and output paths
3. Register it in `KnownPaths`
4. Add it to the current derivation's `input_derivations` with the
   referenced output names

The glue layer MUST detect and reject circular dependencies.

#### Scenario: Chained derivations

- GIVEN derivation A with no inputs, and derivation B with `inputs = [A]`
- WHEN both are processed
- THEN A's drv path appears in B's `input_derivations`, and A is registered
  in KnownPaths before B

#### Scenario: Diamond dependency

- GIVEN A depends on B and C, both B and C depend on D
- WHEN A is processed
- THEN D is converted and registered once, B and C reference it, A references
  B and C

#### Scenario: Circular dependency

- GIVEN A has `inputs = [B]` and B has `inputs = [A]`
- WHEN conversion is attempted
- THEN an error is returned indicating a cycle

### Requirement: Deduplication of shared derivation inputs

When the same derivation appears in multiple `inputs` arrays (diamond
dependency), the glue layer MUST deduplicate by derivation identity
(same ATerm serialization produces the same drv path). The derivation
MUST be converted and registered in KnownPaths only once.

#### Scenario: Shared dependency

- GIVEN derivations B and C both include derivation D in their inputs
- WHEN the full graph is processed
- THEN D is converted once, and both B and C reference the same drv path

### Requirement: Dependency ordering

The glue layer MUST process derivation inputs in dependency order.
The algorithm is recursive descent with memoization via KnownPaths:

1. To convert derivation X, first convert all `Input::Derivation` entries
   in X's `inputs`
2. If a derivation is already in KnownPaths (by structural identity), skip
   conversion and reuse the existing drv path
3. Detect cycles by tracking in-progress conversions (a derivation
   encountered while already being converted indicates a cycle)

### Requirement: Store path computation

The glue layer MUST compute output paths using the BLAKE3-modified
`build_output_path` (for regular derivations) or `build_ca_path` (for
fixed-output derivations) from vendored nix-compat. The derivation path
MUST be computed via `Derivation::calculate_derivation_path` (also BLAKE3).

The `hash_derivation_modulo` MUST be computed (using BLAKE3) for every
derivation and stored in KnownPaths, as it is needed by downstream
derivations that depend on this one.

#### Scenario: Store path determinism

- GIVEN the same derivation parameters
- WHEN converted twice
- THEN identical store paths are produced both times

#### Scenario: Paths differ from Nix

- GIVEN the same derivation parameters
- WHEN crunch computes the path
- THEN it differs from what Nix would produce (BLAKE3 vs SHA-256)

### Requirement: KnownPaths tracking

The glue layer MUST maintain a `KnownPaths` structure tracking:

- Derivation path → `(hash_derivation_modulo, Derivation)` pairs
- Output path → derivation path reverse mapping

This is a reimplementation of snix-glue's `KnownPaths`, adapted to work
without snix-eval's `Value` types.

### Requirement: Environment auto-population

The glue layer MUST automatically add these entries to `Derivation.environment`:

- Each output name → its computed store path (e.g., `"out"` → `"/nix/store/...-hello"`)
- `"system"` → the system string

User-provided `env` entries MUST NOT override auto-populated output paths.
User `env` entries for other keys are added as-is.

This matches the behavior of `builtins.derivationStrict` in Nix.

#### Scenario: Env auto-population

- GIVEN `{ name = "hello", ... }` with `outputs = ["out", "dev"]`
- WHEN converted
- THEN `environment` contains `"out"` and `"dev"` mapped to their computed
  store paths, plus `"system"`

### Requirement: Fixed-output derivation handling

When `fixed_output` is `Some` (the Nickel record had a `fixed_output` field),
the glue layer MUST:

1. Parse `fixed_output.hash` as a Nix hash string (SRI or hex)
2. Read `fixed_output.algo` (string from enum: `"sha256"`, `"sha1"`, etc.)
3. Read `fixed_output.mode` (string from enum: `"flat"` or `"recursive"`)
4. Construct the appropriate `CAHash`
5. Compute the output path via `build_ca_path`

#### Scenario: FOD with sha256

- GIVEN a NickelValue representing:
  ```nickel
  {
    name = "src",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    fixed_output = { hash = "sha256-...", algo = 'sha256, mode = 'flat },
  }
  ```
- WHEN deserialized into `CrunchDerivation` (with `fixed_output: Some(FixedOutput { algo: HashAlgo::Sha256, ... })`) and converted
- THEN the output path is computed as a CA path matching what Nix produces

### Requirement: Error handling

The glue layer MUST return structured errors for:

| Error | When |
|---|---|
| Missing required field | `name`, `builder`, or `system` absent (serde deserialization fails, but Nickel contract should catch first) |
| Invalid hash format | `fixed_output.hash` cannot be parsed |
| Unknown hash algo | `fixed_output.algo` is not a recognized algorithm |
| Circular inputs | Dependency cycle detected |
| Duplicate derivation | Same derivation parameters registered with different hash_derivation_modulo (internal consistency check) |

#### Scenario: Double validation

- GIVEN a NickelValue missing `name` that somehow bypassed the Nickel contract
- WHEN serde deserialization into `CrunchDerivation` is attempted
- THEN deserialization fails with a missing field error (serde provides
  defense in depth — the Rust types enforce the schema independently of
  Nickel contracts)
