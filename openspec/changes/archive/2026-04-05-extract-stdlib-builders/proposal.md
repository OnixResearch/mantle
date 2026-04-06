## Why

The nickel-stdlib spec draws a hard line:

> "crunch MUST be a minimal build engine... It MUST NOT bundle: Builder
> templates or convenience wrappers (bash builder, mkDerivation, etc.),
> Build phase abstractions (unpack/configure/build/install), Input set
> helpers or stdenv equivalents"
>
> "crunch ships the schema; the ecosystem ships the opinions."

The code violates this. `lib/mk_derivation.ncl` is 200+ lines implementing:
- `mkStdenv` with auto-injected base inputs
- `mkDerivation` with unpackPhase, configurePhase, buildPhase,
  installPhase, fixupPhase, checkPhase and default implementations
  (`make -j$NIX_BUILD_CORES`, `./configure --prefix=$out`, etc.)
- `overrideAttrs` with Nix-like `//` update semantics
- `callPackage` for dependency injection
- `mkShell` for dev environments
- `MkDerivationArgs` contract with pname/version/buildInputs/meta/passthru

`lib/lib.ncl` re-exports all of these as `crunch.mkStdenv`,
`crunch.mkDerivation`, `crunch.mkShell`, `crunch.callPackage`.

This is stdenv. It grew into core because it was convenient during
bootstrap development. It should live in a separate Nickel package that
imports the crunch stdlib.

The Derivation contract was also opened (`..`) to accommodate mkDerivation's
extra fields (pname, version, meta, passthru, overrideAttrs). The spec says
it should be closed. The open contract means field name typos pass silently.

## What Changes

1. **Move `mk_derivation.ncl` out of `lib/`** into a separate directory
   (e.g., `packages/stdenv/` or `builders/`). It becomes an external
   Nickel package that `import`s the crunch stdlib.
2. **Remove re-exports** from `lib/lib.ncl` — no more `crunch.mkStdenv`,
   `crunch.mkDerivation`, etc.
3. **Close the Derivation contract** — remove `..`, add
   `addressing_mode` and `sandbox` to the allowed fields.
4. **Update bootstrap .ncl files** to import the builder package
   explicitly instead of using `crunch.mkStdenv`.

## Capabilities

### Modified Capabilities
- `lib/lib.ncl`: exports only Derivation, FixedOutput, enums, validators,
  helpers, fetchers, and `select`
- `lib/derivation.ncl`: closed record contract again
- `builders/` or `packages/stdenv/`: standalone Nickel package with
  mkDerivation, mkStdenv, mkShell, callPackage
- `bootstrap/*.ncl`: imports builder package explicitly

## Impact

- **Files**: moved `lib/mk_derivation.ncl`, modified `lib/lib.ncl`,
  modified `lib/derivation.ncl`, modified `bootstrap/*.ncl`
- **APIs**: `crunch.mkDerivation` etc. no longer exist — users import
  the builder package separately
- **Dependencies**: none (pure Nickel change)
- **Testing**: stdlib tests updated; builder package gets its own tests
