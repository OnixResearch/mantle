# Additive consumer-owned ABI fixture, not a new reference-runner result.
{
  schema = "mantle-spacewasm-host-segments-v1",
  source_revision = "e24cf09355a90497148eb5029fdb8e3400bd63e3",
  wasm_feature_profile = "wasm1",
  fixture_id = "molten-host-segments",
  artifact_path = "fixtures/wasm/molten-host-segments.wasm",
  generator_input_path = "fixtures/wat/molten-host-segments.wat",
  imports = [
    { module = "molten", name = "observe", parameters = ["i32"], results = [] },
    { module = "molten", name = "pause", parameters = [], results = [] },
  ],
  exports = [
    { name = "run", kind = "function", parameters = [], results = [] },
    { name = "total", kind = "global", value_type = "i32", mutable = true },
  ],
  expected_transcript = [
    { event = "observe", value = 7 },
    { event = "pause" },
    { event = "observe", value = 11 },
  ],
  expected_final_total = 18,
  fuel_accounting = "consumer-owned-cumulative-budget-across-resume",
  pause_semantics = "SpaceWasm-host-pause-resume; Wasmtime-normal-host-return",
  comparison_runtime = { name = "wasmtime", version = "41.0.3" },
  execution_status = "not-executed-by-reference-hostless-runner",
  non_claims = [
    "Fixture materialization is not consumer runtime admission.",
    "Wasmtime transcript equality is diagnostic, not identical suspension semantics.",
    "Fuel and segmentation behavior require a separate consumer-owned execution receipt.",
  ],
}
