Task-ID: V4
Covers: r[compiled-eval.cranelift-prototype-subset.unsupported]

Source audit, no fresh compile required:
- `crates/crunch-eval/src/backend.rs` rejects non-empty import paths for `CraneliftPrototypeBackend::eval_to_json` with `cranelift prototype backend does not support import paths`.
- `crates/crunch-eval/src/cranelift_proto.rs::parse_flat_derivation_fields` accepts only `name`, `builder`, `system`, `addressing_mode`, `args`, and `outputs` fields; any other key returns `unsupported field in prototype subset`.
- Existing feature-gated unit test `prototype_rejects_nested_inputs` verifies the unsupported-field path for nested derivation inputs.
- Broader contracts, merges, package sets, and recursive-record semantics remain under `r[compiled-eval.future-semantics]`, not current prototype support.
