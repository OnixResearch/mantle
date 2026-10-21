"""Trusted adapter payload, embedded in the ordinary derivation's arguments.

No acquisition, plugin loading, model import, torch.load, or remote-code path.
The independent reader source is prepended by the Rust adapter.
"""
import importlib.metadata
import json
import os
import platform
import resource
import shutil
import signal
import stat
import subprocess
import sys
import time

REVISION = "ec7f3ac9ab33e46b136eb5ab6a76c4d81f57c7f1"
VERSIONS = {"torch": "2.2.1+cpu", "numpy": "1.26.4", "safetensors": "0.4.5",
            "sentencepiece": "0.2.0", "protobuf": "4.25.5", "blake3": "1.0.4"}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def no_duplicates(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def load_json(path):
    if path.stat().st_size > 1 << 20:
        raise ValueError("JSON byte bound")
    return json.loads(path.read_bytes(), object_pairs_hook=no_duplicates)


def file_digest(path):
    from blake3 import blake3
    h = blake3()
    with path.open("rb") as source:
        while block := source.read(1 << 20):
            h.update(block)
    return h.hexdigest()


def tree_digest(root, bytes_max, roots):
    """Hash canonical JSON rows [member, kind, executable, bytes/hash or target]."""
    from blake3 import blake3
    rows, total = [], 0
    root = Path(root)
    if root.is_symlink() or not root.is_dir():
        raise ValueError("tree root must be an ordinary directory")
    for path in sorted(root.rglob("*")):
        if len(rows) >= 100000:
            raise ValueError("tool tree entry bound")
        member, info = path.relative_to(root).as_posix(), path.lstat()
        if stat.S_ISREG(info.st_mode):
            total += info.st_size
            if total > bytes_max:
                raise ValueError("tool tree byte bound")
            rows.append([member, "file", bool(info.st_mode & 0o111), info.st_size, file_digest(path)])
        elif stat.S_ISDIR(info.st_mode):
            rows.append([member, "directory"])
        elif stat.S_ISLNK(info.st_mode):
            target = path.resolve(strict=True)
            if not any(target == r or r in target.parents for r in roots):
                raise ValueError(f"undeclared tool symlink target: {member}")
            rows.append([member, "symlink", os.readlink(path)])
        else:
            raise ValueError("tool tree contains special file")
    return blake3(b"mantle-model-tree-v1\0" + canonical(rows)).hexdigest()


def shapes():
    return {"model.embed_tokens.weight": [32, 32], "lm_head.weight": [32, 32],
            "model.norm.weight": [32], "model.layers.0.input_layernorm.weight": [32],
            "model.layers.0.post_attention_layernorm.weight": [32],
            **{f"model.layers.0.self_attn.{p}_proj.weight": [32, 32] for p in ["q", "k", "v", "o"]},
            "model.layers.0.mlp.gate_proj.weight": [64, 32],
            "model.layers.0.mlp.up_proj.weight": [64, 32],
            "model.layers.0.mlp.down_proj.weight": [32, 64]}


def tensor_name(name):
    mapping = {"model.embed_tokens.weight": "token_embd.weight", "lm_head.weight": "output.weight",
               "model.norm.weight": "output_norm.weight", "model.layers.0.input_layernorm.weight": "blk.0.attn_norm.weight",
               "model.layers.0.post_attention_layernorm.weight": "blk.0.ffn_norm.weight"}
    for short, target in [("q", "q"), ("k", "k"), ("v", "v"), ("o", "output")]:
        mapping[f"model.layers.0.self_attn.{short}_proj.weight"] = f"blk.0.attn_{target}.weight"
    for short in ["gate", "up", "down"]:
        mapping[f"model.layers.0.mlp.{short}_proj.weight"] = f"blk.0.ffn_{short}.weight"
    return mapping[name]


def inspect_model(plan):
    root = Path(plan["model_root"])
    if any(parent.is_symlink() for parent in root.parents):
        raise ValueError("model ancestor symlink escape")
    expected_files = {f["member"]: f for f in plan["files"]}
    if root.is_symlink() or not root.is_dir() or {p.name for p in root.iterdir()} != set(expected_files):
        raise ValueError("undeclared/missing model file; code, pickle and extra tokenizer files are forbidden")
    for member, identity in expected_files.items():
        path = root / member
        if path.is_symlink() or not path.is_file() or path.stat().st_size != identity["bytes"] or file_digest(path) != identity["blake3"]:
            raise ValueError(f"model content drift: {member}")
    config = load_json(root / "config.json")
    required = {"architectures": ["LlamaForCausalLM"], "model_type": "llama", "hidden_size": 32,
                "intermediate_size": 64, "num_hidden_layers": 1, "num_attention_heads": 4,
                "num_key_value_heads": 4, "vocab_size": 32, "max_position_embeddings": 32,
                "rms_norm_eps": 1e-5, "rope_theta": 10000.0, "hidden_act": "silu",
                "tie_word_embeddings": False, "bos_token_id": 1, "eos_token_id": 2,
                "torch_dtype": "float32"}
    if config != required:
        raise ValueError("unsupported architecture/config/options or code request")
    if load_json(root / "tokenizer_config.json") != {"tokenizer_class": "LlamaTokenizer", "bos_token": "<s>", "eos_token": "</s>", "unk_token": "<unk>", "add_bos_token": True, "add_eos_token": False}:
        raise ValueError("tokenizer config mismatch/code request")
    import sentencepiece as sp
    tokenizer = sp.SentencePieceProcessor(model_file=str(root / "tokenizer.model"))
    if tokenizer.vocab_size() != 32 or [tokenizer.unk_id(), tokenizer.bos_id(), tokenizer.eos_id()] != [0, 1, 2] or [tokenizer.id_to_piece(i) for i in range(3)] != ["<unk>", "<s>", "</s>"]:
        raise ValueError("tokenizer vocabulary/config mismatch")
    index = load_json(root / "model.safetensors.index.json")
    if set(index) != {"metadata", "weight_map"} or not isinstance(index["weight_map"], dict) or set(index["weight_map"]) != set(shapes()):
        raise ValueError("invalid shard index/inventory")
    shards = set(index["weight_map"].values())
    if shards != {n for n in expected_files if n.endswith(".safetensors")}:
        raise ValueError("missing/extra shard")
    # Read the inert container ourselves before importing torch or the converter.
    tensors, total_data = {}, 0
    for shard in sorted(shards):
        raw = (root / shard).read_bytes()
        if len(raw) < 8:
            raise ValueError("truncated safetensors")
        header_bytes = struct.unpack_from("<Q", raw)[0]
        if not 2 <= header_bytes <= 1 << 20 or header_bytes + 8 > len(raw):
            raise ValueError("invalid safetensors header")
        header = json.loads(raw[8:8 + header_bytes], object_pairs_hook=no_duplicates)
        spans = []
        for name, entry in header.items():
            if name == "__metadata__":
                raise ValueError("unexpected safetensors metadata")
            if name not in shapes() or name in tensors or index["weight_map"].get(name) != shard:
                raise ValueError("extra/duplicate/misindexed tensor")
            if set(entry) != {"dtype", "shape", "data_offsets"} or entry["dtype"] != "F32" or entry["shape"] != shapes()[name]:
                raise ValueError("unsupported tensor dtype/shape")
            start, end = entry["data_offsets"]
            if not isinstance(start, int) or not isinstance(end, int) or start < 0 or end - start != math.prod(entry["shape"]) * 4 or 8 + header_bytes + end > len(raw):
                raise ValueError("corrupt tensor extent")
            spans.append((start, end))
            data = raw[8 + header_bytes + start:8 + header_bytes + end]
            values = list(struct.unpack("<" + "f" * (len(data) // 4), data))
            limit = 2.0 if len(entry["shape"]) == 1 else 0.125
            if any(not math.isfinite(x) or abs(x) > limit for x in values):
                raise ValueError("tensor numeric range outside admitted smoke cohort")
            tensors[name] = values
        spans.sort()
        if not spans or spans[0][0] != 0 or any(a[1] != b[0] for a, b in zip(spans, spans[1:])) or spans[-1][1] != len(raw) - 8 - header_bytes:
            raise ValueError("overlap/gap/trailing data in shard")
        total_data += spans[-1][1]
    if set(tensors) != set(shapes()) or index["metadata"] != {"total_size": total_data}:
        raise ValueError("incomplete shard inventory")
    return tensors, tokenizer


def inspect_output(path, tensors, tokenizer, quantized=False):
    r = Reader(path)
    expected = {tensor_name(n): list(reversed(shape)) for n, shape in shapes().items()}
    if {n: t["shape"] for n, t in r.tensors.items()} != expected:
        raise ValueError("GGUF tensor name/shape mismatch")
    meta = {"general.architecture": "llama", "llama.block_count": 1, "llama.embedding_length": 32,
            "llama.context_length": 32, "llama.feed_forward_length": 64,
            "llama.attention.head_count": 4, "llama.attention.head_count_kv": 4,
            "llama.rope.dimension_count": 8, "llama.rope.freq_base": 10000.0,
            "tokenizer.ggml.model": "llama", "tokenizer.ggml.bos_token_id": 1,
            "tokenizer.ggml.eos_token_id": 2, "general.file_type": 7 if quantized else 0}
    if any(r.metadata.get(k) != v for k, v in meta.items()):
        raise ValueError("GGUF metadata mismatch")
    if not math.isclose(r.metadata.get("llama.attention.layer_norm_rms_epsilon", 0.0), 1e-5, rel_tol=1e-6):
        raise ValueError("GGUF RMS numeric metadata mismatch")
    if r.metadata.get("tokenizer.ggml.tokens") != [tokenizer.id_to_piece(i) for i in range(32)]:
        raise ValueError("GGUF tokenizer drift")
    if r.metadata.get("tokenizer.ggml.scores") != [tokenizer.get_score(i) for i in range(32)]:
        raise ValueError("GGUF tokenizer scores drift")
    expected_types = [2 if tokenizer.is_unknown(i) else 3 if tokenizer.is_control(i) else 5 if tokenizer.is_unused(i) else 6 if tokenizer.is_byte(i) else 1 for i in range(32)]
    if r.metadata.get("tokenizer.ggml.token_type") != expected_types:
        raise ValueError("GGUF tokenizer type drift")
    if quantized:
        if not any(t["type"] == 8 for t in r.tensors.values()):
            raise ValueError("quantizer did not produce Q8_0 data")
    else:
        for name, values in tensors.items():
            # Independent HF→GGUF RoPE row permutation for four heads of eight.
            if name.endswith(("q_proj.weight", "k_proj.weight")):
                order = [h * 8 + parity * 4 + pair for h in range(4) for pair in range(4) for parity in range(2)]
                values = [values[row * 32 + col] for row in order for col in range(32)]
            if r.tensors[tensor_name(name)]["type"] != 0 or r.values(tensor_name(name)) != values:
                raise ValueError("converted F32 tensor differs from independently transformed source")
    return r


def execute(command, timeout):
    with open("tool.log", "ab") as log:
        result = subprocess.run(command, stdin=subprocess.DEVNULL, stdout=log, stderr=log, timeout=timeout, check=False)
    if result.returncode:
        raise ValueError(f"declared real tool exited {result.returncode}; see tool.log")


def run_conversion():
    plan = json.loads(os.environ["MANTLE_MODEL_PLAN"], object_pairs_hook=no_duplicates)
    limits = plan["limits"]
    resource.setrlimit(resource.RLIMIT_AS, (limits["address_space_bytes"], limits["address_space_bytes"]))
    resource.setrlimit(resource.RLIMIT_CPU, (limits["cpu_seconds"], limits["cpu_seconds"]))
    resource.setrlimit(resource.RLIMIT_FSIZE, (16 << 20, 16 << 20))
    resource.setrlimit(resource.RLIMIT_NOFILE, (128, 128))
    signal.alarm(limits["wall_seconds"])
    if sys.version_info[:3] != (3, 11, 11) or platform.machine() != "x86_64" or sys.byteorder != "little":
        raise ValueError("unsupported runtime/hardware")
    with open("/proc/cpuinfo", "rb") as cpu_source:
        cpu_bytes = cpu_source.read((2 << 20) + 1)
    if len(cpu_bytes) > 2 << 20:
        raise ValueError("CPU inventory byte bound")
    observed_cpus = set()
    for block in cpu_bytes.decode("ascii").strip().split("\n\n"):
        fields = dict(line.split(":", 1) for line in block.splitlines() if ":" in line)
        fields = {k.strip(): v.strip() for k, v in fields.items()}
        observed_cpus.add((fields.get("model name"), tuple(sorted(fields.get("flags", "").split()))))
    if observed_cpus != {(plan["cpu_model"], tuple(plan["cpu_flags"]))}:
        raise ValueError("CPU model/features differ from declared numeric cohort")
    roots = [Path(t["path"]) for t in plan["tools"]]
    for tree in plan["tools"]:
        if tree_digest(tree["path"], tree["bytes_max"], roots) != tree["blake3"]:
            raise ValueError("tool/runtime/native dependency content drift")
    for package, version in VERSIONS.items():
        if importlib.metadata.version(package) != version:
            raise ValueError(f"runtime dependency cohort mismatch: {package}")
    if plan["converter_revision"] != REVISION:
        raise ValueError("converter revision mismatch")
    # Deliberately no trust_remote_code, arbitrary converter arguments or plugins.
    tensors, tokenizer = inspect_model(plan)
    out = Path(os.environ["out"])
    out.mkdir()
    if shutil.disk_usage(".").free < limits["storage_bytes"]:
        raise ValueError("insufficient declared scratch/output storage")
    converter_entry = ("import runpy,sys,torch; torch.manual_seed(0); torch.set_num_threads(1); "
                       "torch.set_num_interop_threads(1); torch.use_deterministic_algorithms(True); "
                       "sys.argv=sys.argv[1:]; runpy.run_path(sys.argv[0],run_name='__main__')")
    # Two clean, independent tool executions within one admitted generic build.
    results = []
    first_readers = None
    numeric_drift = {}
    for attempt in ["one", "two"]:
        work = Path(attempt)
        work.mkdir()
        f32, q8 = work / "model-f32.gguf", work / "model-q8_0.gguf"
        execute([plan["python"], "-s", "-c", converter_entry, plan["converter"], plan["model_root"], "--outfile", str(f32), "--outtype", "f32", "--eager", "--model-name", "mantle-inert-llama"], limits["wall_seconds"])
        execute([plan["quantizer"], str(f32), str(q8), "Q8_0", "1"], limits["wall_seconds"])
        reference = inspect_output(f32, tensors, tokenizer)
        quantized = inspect_output(q8, tensors, tokenizer, True)
        smoke = compare(reference, quantized)
        results.append({"f32_blake3": file_digest(f32), "q8_blake3": file_digest(q8), "smoke": smoke})
        if attempt == "one":
            first_readers = (reference, quantized)
            shutil.copyfile(f32, out / "model-f32.gguf")
            shutil.copyfile(q8, out / "model-q8_0.gguf")
            inventory = {"f32": {"metadata": reference.metadata, "tensors": reference.tensors},
                         "q8_0": {"metadata": quantized.metadata, "tensors": quantized.tensors}}
        else:
            for label, first, second in zip(["f32", "q8_0"], first_readers, [reference, quantized]):
                numeric_drift[label] = max(abs(a - b) for name in first.tensors
                                           for a, b in zip(first.values(name), second.values(name)))
        if sum(p.stat().st_size for p in Path(".").rglob("*") if p.is_file()) > limits["storage_bytes"]:
            raise ValueError("declared storage bound exceeded")
    provenance = {"schema": "mantle-model-conversion-provenance-v1", "plan": plan,
                  "plan_blake3": os.environ["MANTLE_MODEL_PLAN_BLAKE3"],
                  "runtime_versions": VERSIONS, "python": platform.python_version(),
                  "reader": "independent embedded mantle-gguf-reader-v1; exact source bound in derivation args",
                  "reader_blake3": os.environ["MANTLE_MODEL_READER_BLAKE3"],
                  "runner_blake3": os.environ["MANTLE_MODEL_RUNNER_BLAKE3"],
                  "native_build_required": "GGML_NATIVE=OFF; GGML_AVX=OFF; GGML_AVX2=OFF; GGML_FMA=OFF; GGML_F16C=OFF; GGML_CUDA=OFF; GGML_VULKAN=OFF; GGML_BLAS=OFF; GGML_OPENMP=OFF",
                  "runs": results, "clean_bytes_equal": all(results[0][k] == results[1][k] for k in ["f32_blake3", "q8_blake3"]),
                  "measured_numeric_nondeterminism": numeric_drift,
                  "non_claims": ["No inference quality evaluation", "No semantic equivalence proof", "No cross-hardware reproducibility assertion", "No training, network acquisition, or credential handling"]}
    (out / "inventory.json").write_bytes(canonical(inventory) + b"\n")
    (out / "provenance.json").write_bytes(canonical(provenance) + b"\n")
    shutil.copyfile("tool.log", out / "tool.log")
    checksums = {p.name: file_digest(p) for p in sorted(out.iterdir()) if p.is_file()}
    (out / "checksums.blake3.json").write_bytes(canonical(checksums) + b"\n")


if __name__ == "__main__":
    run_conversion()
