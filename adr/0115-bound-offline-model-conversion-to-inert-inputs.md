# ADR 0115: Bound offline model conversion to inert inputs

- **Status:** Proposed
- **Date:** 2026-10-18

## Context

Issue #12 requires declared CPU GGUF conversion and quantization, with exact
weights/tokenizer/converter/runtime/native/config identities, real offline
execution and independent bounded validation. Model packages can contain
pickle objects, plugin code and tokenizer configuration that requests remote
execution. A converter revision label or executable path is not content
identity, and provenance cannot establish language-model quality.

## Decision drivers

- Use the existing signed input/output admission and generic derivation
  scheduler/sandbox/store, with no model-specific execution authority.
- Reject code-bearing or unsupported model inputs before invoking expensive
  native/Python conversion.
- Separate a real pinned-tool observation from mocked contracts or a driver
  whose prerequisites have not been provisioned.
- Independently inspect values and format structure instead of asking the
  converter's own reader to confirm its own output.
- Record measured clean-run byte and numeric drift rather than claiming
  universal determinism or canonicalizing away differences.

## Decision

Use no_std `crunch-model-conversion-core` for closed plan admission and std
`crunch-model-conversion` for ordinary derivation lowering. The owned
`mantle-model-conversion` authoring executable does not execute builds.
`lib/model_conversion.ncl` contracts its output as an input-addressed native
derivation; the normal Mantle build path remains the only execution/trust
boundary.

Select unchanged llama.cpp revision
`ec7f3ac9ab33e46b136eb5ab6a76c4d81f57c7f1` (b4514), CPython 3.11.11,
Torch 2.2.1+cpu, NumPy 1.26.4, safetensors 0.4.5, SentencePiece 0.2.0,
protobuf 4.25.5 and BLAKE3 1.0.4. Require independently reviewed complete
source/installed-tool/native/runtime tree identities in addition to version
labels. The finite model cohort is one small Llama/SentencePiece decoder,
F32 inert safetensors and GGUF v3 F32 followed by real Q8_0 quantization.
ONNX, model code, pickle, arbitrary options and calibration are rejected, not
advertised as supported backends.

Bind exact CPU model/features and declare memory, storage, per-process CPU
and total wall bounds. Verify the full inert shard/index/tokenizer/config
inventory before real converter execution. An embedded, independently
implemented stdlib-only GGUF reader checks tensor/metadata inventories,
source-to-F32 tensor transforms, Q8_0 dequantization and finite bounded tensor
projection errors. Its exact source and execution payload are bound into
ordinary derivation arguments/environment. Two clean real conversion and
quantization process pairs produce byte comparisons and direct per-element
numeric differences in provenance. These observations make no inference
quality or semantic equivalence claim.

The real proof driver must import explicitly pinned tools and generated
inert sources through normal signed admission and invoke actual generic
`mantle build`. Missing independently reviewed tool/runtime/native closure
pins are a proof prerequisite failure; no mock converter, ambient package
lookup or fabricated successful receipt is permitted.

## Consequences

The fragment is deliberately small and rejects otherwise legitimate model
families/dimensions/options. Supporting another cohort requires independently
validated admission, tool pins, reader coverage and numerical bounds rather
than merely adding a converter option. Full tree hashing incurs bounded
execution cost but keeps installed native/Python support bytes visible in
identity. Acquiring the real tool cohort and checking source-to-binary build
correspondence remain separate operator responsibilities. This ADR does not
claim the selected tool closure has been provisioned or that the real-tool
proof has been observed.
