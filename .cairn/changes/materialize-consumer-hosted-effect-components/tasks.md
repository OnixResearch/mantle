# Tasks

## Source implementation

- [x] [serial] Add closed effects-profile selection and exact checker-owned seed dispatch. r[mantle.wasm_component.effects_cohort] r[mantle.wasm_component.profile_selection]
- [x] [serial] Add identity-bound typed consumer-host runtime requirement with standalone default. r[mantle.wasm_component.consumer_host_requirement]
- [x] [serial] Reject contradictory runtime invocation/stdout and mismatched host requirements. r[mantle.wasm_component.consumer_host_requirement]
- [x] [serial] Preserve static stages and exact non-WASI host-import inspection. r[mantle.wasm_component.consumer_host_requirement]
- [x] [serial] Prepare genuine pinned-toolchain source/WIT fixture without generated-text rewrites. r[mantle.wasm_component.effect_fixtures]
- [x] [serial] Add behavior-level runtime requirement, empty closure, and selection denials. r[mantle.wasm_component.standalone_empty_closure]

## Integration acceptance

- [x] [serial] Publish reviewed checker and repin its actual immutable revision/catalog identities. r[mantle.wasm_component.effects_cohort] Published Octet `abb6082062251944a40093a274df3d35331fa168`; coherent generated lock and realized cohort verified.
- [x] [serial] Generate final toolchain cohort, fixture bindings, and bounded production artifacts. r[mantle.wasm_component.effect_fixtures] Both genuine fixed-memory CLI bundles passed report and complete Structural+Bytes/source-copy verification; actual compiled memories declare 32 initial/maximum pages and adapter tables declare 1 initial/maximum element. Earlier growable-declaration bundles remain retained counterexamples.
- [x] [serial] Run source tests, standalone regression, exact effect pipeline, complete Structural+Bytes verification, and strict Cairn validation. r[mantle.wasm_component.standalone_empty_closure] Focused crate and real pinned-toolchain CLI checks passed; repository-wide shipping gates remain separately required before implementation publication.
- [ ] [serial] Record separately owned real Molten actor and system-extension observations before consumer activation, retaining bounded execution receipts. r[mantle.wasm_component.consumer_host_requirement]

No integration check is marked complete merely from source edits. Immutable
implementation publication requires the recorded producer gates; archival
additionally requires separately owned real-consumer observations after the
consumer's immutable producer repin. Static materialization never substitutes
for consumer execution evidence.
