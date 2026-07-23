# Validation evidence: full-source provider admission

Date: 2026-07-19

## Implementation identity

The provider implementation was committed before the admitted build and external admission sequence. The committed implementation identity is `0acc862a` (`replace bootstrap delegation with a source-built provider chain`). `bootstrap/seed.ncl` still selected the legacy provider while this evidence was produced.

## Committed-source construction and closure

From committed implementation source, Mantle exported and verified the complete materialized fixed-fetch closure for `bootstrap/seed-full-toolchain.ncl`, then passed the offline source preflight before building the provider. The durable source authority is:

```text
source_closure_records=51
source_closure_payload_bytes_approx=544.67 MiB
source_closure_manifest_blake3=2bd4fb6404fd0b3ac208fb1d8f9d2e1f28aa164a48f7470cfe199f17456a70cb
readiness=Ready
missing=0
stale=0
unsupported=0
untrusted=0
```

The closure contains both pinned sbase revisions and contains no `musl.cc`, `seed-legacy.ncl`, or `/mantle/store/` record metadata. Its payload remains external; the independently supplied manifest BLAKE3 binds it to this evidence.

The committed-source provider build completed at:

```text
provider_store_name=q366f9q6n1silgmv8w0c2ypxxw4p4yw3-full-source-seed-toolchain
provider_log=m5grcxvz6kpl5z48s44hn2xgwj8d79aq-full-source-seed-toolchain.drv.log
provider_log_status=success
```

The builder's bounded runtime admission covered C and C++ static/dynamic compilation and execution, assembly/link execution, archive indexing, object inspection/transformation, CRT/libc/libgcc/libstdc++ surfaces, malformed C/C++/assembly rejection, and undefined-symbol rejection. The normalized shared runtimes retain basename `DT_NEEDED` entries and reject absolute store-path dependencies.

## Independent identity and external admission

An independent complete Unix-mode tree hash was repeated twice over the final provider with identical results:

```text
provider_tree_entries=1242
provider_file_bytes=154002038
provider_output_blake3=f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f
provider_metadata_blake3=107a4a6d7d17b7da362b4bb64830cbb4ffc64ba3ce68772aabfe05584781be23
```

`mantle bootstrap full-source-provider-admit` then observed the materialized provider and complete source bundle, matched both independently supplied BLAKE3 identities, executed all 25 positive/rejection runtime steps, and wrote the create-new report `evidence/full-source-provider-admission.json`. Its admitted facts are:

```text
schema=mantle-full-source-provider-admission-v2
status=admitted
provider_id=full-source-v1
required_tool_count=18
required_runtime_count=10
runtime_smoke_step_count=25
provider_output_blake3=f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f
source_closure_manifest_blake3=2bd4fb6404fd0b3ac208fb1d8f9d2e1f28aa164a48f7470cfe199f17456a70cb
provider_metadata_blake3=107a4a6d7d17b7da362b4bb64830cbb4ffc64ba3ce68772aabfe05584781be23
```

Admission rejects digest mismatch, incomplete or planned source records, state-pinned source metadata, legacy closure markers, TinyCC delegation, generated stubs, missing compiler internals, missing C++/runtime surfaces, unsafe symlinks, host fallback markers, malformed metadata, and rejected-command output leftovers.

## Focused validation before selector promotion

Pueue task `280`:

```text
nix develop -c cargo test -p mantle --test bootstrap_eval

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.38s
```

Pueue task `281`:

```text
nix develop -c cargo test -p mantle --bin mantle full_source_provider::tests

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1585 filtered out; finished in 0.01s
```

`git diff --check` also passed before the evidence commit.

## Adversarial audit and claim boundary

Static metadata, executable bits, version output, diagnostic private overlays, source-probe roots, and downstream compiler success were not accepted as provider admission. The surviving mechanism binds a real runtime-tested provider tree and a fully materialized source closure to independent BLAKE3 identities after the implementation commit. Selection remains a separate change so a failed or incomplete candidate cannot silently fall back or become bootstrap authority.

This evidence proves the recorded provider construction, bounded runtime surfaces, and source/output identity on the recorded x86_64-linux orchestration boundary. It does not prove compiler correctness, bootstrap-seed correctness, independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility. The selected-provider authenticated stage0 → stage1 → stage2 fixed-point proof remains pending.
