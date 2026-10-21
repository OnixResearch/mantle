# Offline CPU model conversion

## Supported fragment and status

Issue [#12](https://github.com/OnixResearch/mantle/issues/12) starts with
CPU-only Llama/SentencePiece safetensors → little-endian GGUF v3 F32 → Q8_0.
ONNX, pickle/PyTorch checkpoints, remote model code, adapters, training, gated
credentials, acquisition inside a build, calibration/importance matrices,
arbitrary converter arguments and other quantization encodings are rejected.

The deliberately finite initial numerical cohort has one Llama decoder block,
32 hidden dimensions, 64 intermediate dimensions, four attention/KV heads,
32 vocabulary entries and 32 context positions. Input matrices must be finite
F32 with absolute values at most 0.125; norm vectors at most 2.0. This is an
inert format/conversion fixture, **not a useful language model**. It does not
establish support for arbitrary production Llama models. Every config field
is checked, including special-token IDs, activation, RoPE and tied-weight
policy; unknown model/config/tokenizer fields and extra files are rejected.

The source includes a real-tool driver, not a successful execution receipt.
No full pinned conversion/runtime closure has been checked in or claimed to
have run. Acceptance requires supplying the independently reviewed content
lock described below and actually running the proof. Missing tool trees fail
rather than falling back to host tools, a mock converter or downloaded models.

## Exact selected cohort

- llama.cpp revision `ec7f3ac9ab33e46b136eb5ab6a76c4d81f57c7f1`, upstream
  [`b4514`](https://github.com/ggml-org/llama.cpp/tree/ec7f3ac9ab33e46b136eb5ab6a76c4d81f57c7f1),
  providing the unchanged `convert_hf_to_gguf.py`, its `gguf-py` package and
  the real `llama-quantize` executable.
- CPython **3.11.11**, CPU PyTorch **2.2.1+cpu**, NumPy **1.26.4**,
  safetensors **0.4.5**, SentencePiece **0.2.0**, protobuf **4.25.5**,
  Python BLAKE3 **1.0.4**. The runner checks these versions in addition to
  checking the content identities of their complete trees.
- A self-contained native quantizer built for `x86_64-linux` with
  `GGML_NATIVE=OFF`, `GGML_AVX=OFF`, `GGML_AVX2=OFF`, `GGML_FMA=OFF`,
  `GGML_F16C=OFF`, `GGML_CUDA=OFF`, `GGML_VULKAN=OFF`, `GGML_BLAS=OFF`,
  `GGML_OPENMP=OFF`. These are required acquisition/build-review assumptions,
  **not flags inferred or proven from executable bytes by the adapter**.
- All native dynamic loaders/libraries, Python extension dependencies and
  converter support modules must be in the explicit admitted closure. No
  host `/lib`, host Python site packages, `PATH`, or dependency resolution is
  available during conversion. Extra Python dependency versions are bound
  by complete tree content, not allowed to float at execution time.
- The exact CPU model string and sorted feature set of every processor must
  match the plan's `cpu_model` and `cpu_flags`. Heterogeneous CPU inventories
  are unsupported. Little-endian x86_64 and no accelerators are required.
  Reproducibility is not generalized to other CPU models, microcode, operating
  systems or toolchain builds.

The revision and version labels are descriptive constraints, **not content
identity**. The reviewed lock must bind actual source, installed Python,
native tool and complete runtime/native dependency tree bytes independently.
Rebuilding that cohort with another compiler/library produces another lock
and action identity even if all version labels remain equal. Acquiring or
building those trees, license review and source-to-binary correspondence
review happen outside this network-denied adapter.

## Authoring and admission

`crunch-model-conversion-core` is no_std + alloc. It rejects unsupported
options, malformed members/identities, undeclared executable paths and
out-of-range resource declarations. `crunch-model-conversion` embeds the
trusted execution payload and independent reader, then emits an ordinary
derivation. There is no model scheduler, new resolver, trust database or
alternate execution route.

The Rust core consumes `Plan` in `validate` and returns that same owned plan
after admission. `crunch_model_conversion::lower` likewise consumes the plan,
so validation does not require cloning its model or tool declarations. This
ownership boundary does not change the plan JSON schema, supported format,
calibration rejection, numerical checks, or resource limits.

`mantle-model-conversion STORE_PREFIX` reads at most 1 MiB of plan JSON from
stdin and writes derivation JSON to stdout. The closed `Plan` type is the
wire contract; all fields are required:

| Field | Contract |
|---|---|
| `schema` | `mantle-model-conversion-v1` |
| `converter_revision` | exact revision above |
| `system`, `hardware` | `x86_64-linux`, `x86_64-cpu-little-endian-no-accelerator` |
| `cpu_model`, `cpu_flags` | exact model and sorted unique feature list from every `/proc/cpuinfo` processor |
| `model_root` | admitted logical store directory, optionally a directory member of one store object |
| `files` | every config/tokenizer/index/shard as `{member, blake3, bytes}` |
| `tools` | complete root set as `{path, blake3, bytes_max}`; no nested root aliases |
| `python`, `python_site`, `converter`, `quantizer` | exact absolute members of declared tool roots; site is one closed Python site-package directory |
| `format`, `quantization` | `gguf-v3-f32`, `Q8_0` |
| `calibration`, `seed`, `threads` | `[]`, `0`, `1` |
| `limits` | `address_space_bytes` 1–8 GiB, `storage_bytes` 256 MiB–2 GiB, `cpu_seconds` 1–300, `wall_seconds` 1–600 |

Model members consist only of `config.json`, `tokenizer.model`,
`tokenizer_config.json`, `model.safetensors.index.json` and named
`model-*.safetensors` shards. At most 32 files and 64 MiB are admitted. The
index's names/shard mapping/total size, tensor dimensions, header extents,
non-overlap, no gaps, no trailing data and numeric bounds are checked before
invoking expensive conversion. Duplicate JSON keys and tensor names fail.
File checks include byte count, BLAKE3 and rejection of model symlinks.

Tool trees use the **mantle-model-tree-v1** preimage: BLAKE3 over ASCII prefix
`mantle-model-tree-v1` plus one NUL byte plus canonical UTF-8 JSON (sorted keys,
compact separators, ASCII escaping) of lexically sorted relative entry rows.
Regular files use `[member,"file",executable_boolean,byte_count,blake3_hex]`,
directories `[member,"directory"]`, symlinks `[member,"symlink",literal_target]`.
Only these entry kinds are supported; the root itself is an ordinary
directory. Every resolved symlink target must exist under a declared tool
root. Files are streamed while hashing; each root is bounded by
`bytes_max` (at most 8 GiB) and 100,000 entries. The reviewed lock must be
prepared independently; deriving a fresh digest from unreviewed executable
bytes at proof time is not approval.

Use the helper only after lowering:

```nickel
(import "model_conversion.ncl").derivation (import "lowered.json")
```

The helper sets native sandbox, `x86_64-linux`, and input addressing. The
existing Mantle generic build admission verifies declared store inputs and
signs outputs. Paths are validated against the exact selected logical store
prefix. Merely having a path/version/digest-shaped string is not store trust.
The source/tool/options/hardware plan plus the exact reader/runner payload
are in derivation inputs/arguments/environment and therefore build identity.
A nonempty calibration declaration fails early rather than affecting nothing.

## Output and numerical/reproducibility claims

Outputs include `model-f32.gguf`, `model-q8_0.gguf`, `inventory.json`,
`checksums.blake3.json`, `provenance.json` and real `tool.log`.

The independent **stdlib-only** reader in `gguf_reader.py` imports neither
llama.cpp's `gguf` reader nor Torch/NumPy. Its exact source BLAKE3 and the
runner source BLAKE3 are recorded and bound to derivation arguments. It checks
GGUF v3 encoding/extents, tensor names/shapes, metadata and tokenizer values,
and supports only F32 and Q8_0. F32 tensor values must equal independently
transformed source values, including the Llama Q/K RoPE row permutation.
Q8_0 values are independently dequantized from binary blocks.

Each admitted build launches two clean converter/quantizer process pairs in
separate scratch directories. Seed zero, deterministic Torch algorithms,
one intra/inter-op thread, one native quantizer thread, fixed locale/timezone,
Python hash seed, offline Hugging Face mode and bounded numeric settings are
explicit. Both GGUF artifacts are compared by BLAKE3. Direct maximum
per-element differences between corresponding clean runs are recorded, not
inferred from a same maximum-error summary. Byte inequality and measured
numeric nondeterminism remain visible in provenance; they are not relabeled
as equivalence or hidden by rounding/canonicalizing output tensors.

The numerical smoke checks every dequantized value for finiteness, absolute
F32-to-Q8_0 tensor error ≤0.002 and a fixed bounded projection error ≤0.02.
This is distinct from inference, perplexity, quality evaluation or semantic
equivalence. It does not prove model usefulness or global reproducibility.

Limits are enforced by address-space/CPU/file-size/open-FD resource limits,
a total wall alarm, a 16 MiB per-file hard cap, fixed process count and bounded
input/output inventories. CPU seconds apply per process; the wall alarm bounds
the entire adapter operation, including tool-tree hashing. A declared storage
reservation is checked before tools, and scratch usage is checked after each
pair. For this fixed shape and four process invocations the file-count/size
bounds fit inside the minimum storage declaration. The generic sandbox owns
teardown of all descendants; this adapter does not create an independent
resource scheduler.

## Real proof command and required observations

After the parent integrated build, run the driver detached on spacious Linux
scratch. Supply absolute prebuilt executable paths, a fresh run directory,
and the independently reviewed full plan/tool lock plus its independently
recorded BLAKE3:

```sh
python3 scripts/prove-model-conversion.py \
  --mantle "$MANTLE" --lowerer "$MODEL_LOWERER" \
  --nix "$NIX" --b3sum "$B3SUM" \
  --cohort-lock "$REVIEWED_COHORT_LOCK" \
  --cohort-lock-blake3 "$REVIEWED_COHORT_LOCK_BLAKE3" \
  --run-dir "$FRESH_MODEL_PROOF_DIR"
```

The driver does no fetching or tool building. It requires every tool closure
already present, verifies the lock BLAKE3 and transitive Nix closure equality,
generates a public-domain two-shard fixture without training or Torch,
imports source/tool inputs through a locally signed cache and Mantle's normal
`store pull`, lowers the plan and invokes **actual `mantle build`**, then
verifies the exact output's NAR/signatures with a matched one-path check.
Nix is used only for source acquisition/admission before conversion; it is
not an execution dependency of a model build.

`proof.json` must show actual baseline conversion/quantization with the pinned
reader and numeric measurements, identical-plan cache reuse, changed-weight
identity **and converted bytes**, changed resource/tool-content identities,
and rejected missing/corrupt shards, tokenizer mismatch, unsupported
architecture/options/revision, undeclared code/pickle and calibration.
Execution negatives must reach their expected saved-log admission diagnosis;
a failed prerequisite does not count as a negative control. Clean byte and
direct numeric comparisons are in the admitted output provenance. A blocker
records `status=blocked`; it is not successful evidence. No model is uploaded.

The ordinary focused checks for parent integration are:

```sh
cargo test -p crunch-model-conversion-core --lib
python3 tests/fixtures/model-conversion/test_reader.py
```

These boundary tests are not the real-tool proof and cannot satisfy issue #12
by themselves.
