# Source input binding corrections before the next cold run

## Observed failure (proof v26, remote leviathan)

The v26 cold proof advanced through the Rust stage (5h33m) and then failed
source admission of `bootstrap/source-built/mantle-stage1.json`:

```text
source_built_fixed_point: blocked: source input validation at bootstrap/source-built/mantle-stage1.json: step unit-XXXXX-link.native-entry references unknown input mantle-source
```

## Defects and fixes

1. `source_built_rust_programs_translation::append_link_response` emitted a
   native entry prepare step whose `cwd` word names the authenticated compiler
   source but whose declared inputs did not bind that name. The step now binds
   every input name its own `cwd` word addresses from the values it already
   receives, and fails admission when the name is absent from them.
   Commit `9c04d710a`.
2. Mantle stage authoring attached shared original environment words (for
   example the produced Rust provider) to steps that declared only their own
   operands. `Stage::finish` now binds every name an operation addresses by
   name from `Stage::compiler_inputs()`, so a step declares exactly the
   artifacts its words consume. Commit `16ee6bca2`; unit coverage for the
   referenced-name inventory in commit `0475a09f8`.

## Preparation and preflight evidence

Prepared cohort `prepared-cohort-v20` with the corrected binary:

```text
output_profile_blake3 = 35a3574b2cb3215ae4a87e83e7f9932c417bb7e3c90bdce2932946f1ecc82743
```

An independent scan of every program in the cohort (steps whose operation
addresses a name absent from its declared inputs) reported:

```text
unknown-input refs 0
```

The previous cohort (`prepared-cohort-v19`) reported 2230 references, all
`rust-provider` on script steps in `mantle-stage1` and `mantle-stage2`.

The corrected binary and cohort were transferred as `bin/mantle-v33` and
`profile-v20/`; proof `v27` was launched against them.

## Non-claims

This note records authoring corrections, a prepared cohort identity, and a
static input-binding scan. It is not a fixed-point proof, bootstrap, or release
claim; only the captured `proof-v27` run and its reports can support one.
