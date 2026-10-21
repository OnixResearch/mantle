#!/usr/bin/env python3
"""Generate public-domain synthetic inert Llama weights/tokenizer, without torch.

No text corpus, downloaded weights, licensed vocabulary, gated access or training.
Run with the declared CPython/SentencePiece/BLAKE3 cohort, before offline builds.
"""
import argparse
import json
from pathlib import Path
import struct


def dump(path, value):
    path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n")


def generate(root, changed=False):
    from sentencepiece import sentencepiece_model_pb2 as pb
    root.mkdir()
    model = pb.ModelProto()
    model.trainer_spec.model_type = pb.TrainerSpec.UNIGRAM
    model.trainer_spec.vocab_size = 32
    model.trainer_spec.unk_id, model.trainer_spec.bos_id, model.trainer_spec.eos_id = 0, 1, 2
    model.trainer_spec.pad_id = -1
    model.normalizer_spec.name = "identity"
    model.normalizer_spec.add_dummy_prefix = True
    model.normalizer_spec.remove_extra_whitespaces = True
    model.normalizer_spec.escape_whitespaces = True
    for i, token in enumerate(["<unk>", "<s>", "</s>", "▁"] + list("abcdefghijklmnopqrstuvwxyz") + [".", ","]):
        piece = model.pieces.add()
        piece.piece, piece.score = token, -float(i)
        piece.type = pb.ModelProto.SentencePiece.UNKNOWN if i == 0 else pb.ModelProto.SentencePiece.CONTROL if i < 3 else pb.ModelProto.SentencePiece.NORMAL
    (root / "tokenizer.model").write_bytes(model.SerializeToString(deterministic=True))
    config = {"architectures": ["LlamaForCausalLM"], "model_type": "llama", "hidden_size": 32,
              "intermediate_size": 64, "num_hidden_layers": 1, "num_attention_heads": 4,
              "num_key_value_heads": 4, "vocab_size": 32, "max_position_embeddings": 32,
              "rms_norm_eps": 1e-5, "rope_theta": 10000.0, "hidden_act": "silu",
              "tie_word_embeddings": False, "bos_token_id": 1, "eos_token_id": 2,
              "torch_dtype": "float32"}
    dump(root / "config.json", config)
    dump(root / "tokenizer_config.json", {"tokenizer_class": "LlamaTokenizer", "bos_token": "<s>", "eos_token": "</s>", "unk_token": "<unk>", "add_bos_token": True, "add_eos_token": False})
    definitions = {"model.embed_tokens.weight": [32, 32], "lm_head.weight": [32, 32], "model.norm.weight": [32],
                   "model.layers.0.input_layernorm.weight": [32], "model.layers.0.post_attention_layernorm.weight": [32],
                   **{f"model.layers.0.self_attn.{p}_proj.weight": [32, 32] for p in ["q", "k", "v", "o"]},
                   "model.layers.0.mlp.gate_proj.weight": [64, 32], "model.layers.0.mlp.up_proj.weight": [64, 32],
                   "model.layers.0.mlp.down_proj.weight": [32, 64]}
    weight_map, total = {}, 0
    ordered = sorted(definitions.items())
    for part, selection in enumerate([ordered[:6], ordered[6:]], 1):
        shard = f"model-{part:05}-of-00002.safetensors"
        header, payload = {}, bytearray()
        for position, (name, shape) in enumerate(selection):
            n = 1
            for d in shape:
                n *= d
            values = [1.0] * n if len(shape) == 1 else [((i * 17 + position * 13) % 257 - 128) / 1024 for i in range(n)]
            if changed and name == "lm_head.weight":
                values[0] += 1 / 1024
            data = struct.pack("<" + "f" * n, *values)
            header[name] = {"dtype": "F32", "shape": shape, "data_offsets": [len(payload), len(payload) + len(data)]}
            payload.extend(data)
            weight_map[name] = shard
        encoded = json.dumps(header, sort_keys=True, separators=(",", ":")).encode()
        encoded += b" " * (-len(encoded) % 8)
        (root / shard).write_bytes(struct.pack("<Q", len(encoded)) + encoded + payload)
        total += len(payload)
    dump(root / "model.safetensors.index.json", {"metadata": {"total_size": total}, "weight_map": weight_map})


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("output", type=Path)
    p.add_argument("--changed-weights", action="store_true")
    a = p.parse_args()
    generate(a.output, a.changed_weights)


if __name__ == "__main__":
    main()
