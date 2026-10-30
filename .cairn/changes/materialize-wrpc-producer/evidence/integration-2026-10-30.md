# Integration run, 2026-10-30

Executed in `/home/brittonr/.cargo-target/mantle-pilot-materialization-20261029`
on the tree rebased onto upstream `main` `9a618c27362a7f2e0f8dcf61913e598e8b022523`.
Nix ran with `builders =`, `max-jobs = 2`, `cores = 4`.

## Defects found and repaired in the authored producer

1. `wkg wit fetch` tried the empty local registry for `wrpc-examples:hello`
   (`Error: IO error: No such file or directory`). The guest workspace now
   carries `wkg.toml` path overrides to `wit-deps/`, outside any role's
   `wit/deps` (which `wkg` wipes and rewrites). The registry stays empty.
2. Offline guest compilation failed with ``linker `cc` not found``: the
   `wit-bindgen` macro crate brings host build scripts and proc-macros, and the
   pipeline sandbox has no host linker. The guests are now dependency-free
   `no_std` crates, following the upstream `fixtures/wasm-component-effects`
   pattern. The source step generates bindings with the pinned toolchain
   `wit-bindgen` CLI (0.55.0), and `guest_rt.rs` supplies the allocator,
   `cabi_realloc`, `memcmp` and panic handler.
3. `wit-bindgen-wrpc` needed `--generate-all` for the imported interface.
4. The bundle step had no writable `HOME`, so Mantle failed with
   `materializing embedded Nickel stdlib: Permission denied`.

## Executed results

```sh
nix build --no-link --print-out-paths --keep-going -L \
  .#wrpc-reference-components .#wrpc-telemetry-reference-components
```

Exit 0:
`/nix/store/p2drbz2qbsh0f1ksmviajlx36888z55n-mantle-wrpc-components-aa01409ed38bf7a978570ff567c4475376925df6`,
`/nix/store/nrxfyrd9w38q59y02kyvi52gifmsh111-mantle-wrpc-telemetry-components-e5262a84044aff064c9599eebc05013bb3432795`.

A manual run with the same toolchain
(`/nix/store/rxkdd20n7c510z6n0jd4s4f15zf75lfb-mantle-wasm-component-toolchain-v1`)
composed the client and server with `wac compose`. `wasmtime run --invoke 'run()'`
printed `42`. The composed telemetry emitter and sink printed `42` for `run(0)`.
For `run(1)`, the comparison sink trapped on its non-empty-attribute assertion.
This comparison smoke is not remote RPC evidence.

```sh
nix build --no-link --print-out-paths --keep-going -L \
  .#wrpc-reference-bundle .#wrpc-telemetry-reference-bundle
```

Exit 1. The Mantle pipeline stopped at authoritative blockers:

- `wrpc-reference-bundle`, role `server`:
  `{"code":"octet-report-not-admitted","stage_key":"octet-validation"}`. Octet
  returned a non-pass report for the guest under the toolchain's default
  artifact profile.
- `wrpc-telemetry-reference-bundle`, role `telemetry`:
  `{"code":"wasi-virt-failed","stage_key":"virtualization"}`, with the message
  `no dependencies of component .../composed.wasm were found`. The `no_std`
  emitter has no WASI imports, so deny-all `wasi-virt` has nothing to compose.

No bundle output, member identity or publication identity exists yet.

## Remaining decisions

- Octet admission of the generated-binding guest source needs either a guest
  change that satisfies the selected profile or a different reviewed profile.
  The retained Octet report must be inspected first. This is an Octet-owned
  policy decision; Mantle must not reinterpret the report.
- The pipeline needs a defined outcome for components with no WASI imports.
  It must either skip deny-all virtualization with recorded evidence, or the
  manifest must declare that virtualization is not applicable.
