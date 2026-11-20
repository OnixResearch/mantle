# Tasks: Admit Octet's current producer layout in the verification bridge

## Phase 1: Receipt-derived provider profile locator

- [x] [serial] T1.1 Replace the `PROVIDER_PROFILE_PATH` constant with `octet_provider_profile_locator` and use it in `check_octet_provider_profile` and the checked-material list. r[verification_bridge.octet_producer_layout]
- [x] [serial] T1.2 Call the core locator from the shell's `octet_material_paths` instead of repeating the rule. r[verification_bridge.octet_producer_layout]
- [x] [serial] T1.3 Add stable `MissingProviderProfileLocator` and `AmbiguousProviderProfileLocator` variants and their diagnostics. r[verification_bridge.octet_producer_layout]

## Phase 2: Versioned tool-graph admission

- [x] [serial] T2.1 Accept the `version: 4` `derivations` map with store basenames, static outputs, empty dynamic outputs, and a closure-contained dependency set. r[verification_bridge.octet_producer_layout]
- [x] [serial] T2.2 Keep the legacy map rules unchanged and reject mixed nodes, unsupported versions, and non-empty dynamic outputs with stable variants. r[verification_bridge.octet_producer_layout]

## Phase 3: Verification

- [x] [serial] T3.1 Add positive and negative provider-locator tests (recorded fixture, real-layout copy, missing, duplicate, digest mismatch, escape). r[verification_bridge.octet_producer_layout]
- [x] [serial] T3.2 Add positive and negative graph tests (legacy, synthetic v4, mixed node, version 5, absent root, outside closure, dynamic output). r[verification_bridge.octet_producer_layout]
- [x] [serial] T3.3 Run focused tests, clippy, fmt, and `./scripts/check-no-std-core.sh`; record the transcripts. r[verification_bridge.octet_producer_layout]
- [x] [serial] T3.4 Record ADR 0167, update the bridge doc, and run `cairn validate` plus the proposal, design, and tasks gates. r[verification_bridge.octet_producer_layout]
- [x] [serial] T3.5 Run `octet-job` on the realized pilot output and record the observation. r[verification_bridge.octet_producer_layout]
