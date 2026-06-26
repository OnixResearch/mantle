# Provider-backed proof rerun after optional sandbox-shell compile default

Task-ID: V3
Covers: r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]

## Command provenance

The real provider-backed fixed-point proof was launched with `SNIX_BUILD_SANDBOX_SHELL` unset and wrote the proof bundle at:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-snix-sandbox-shell-2026-06-26
```

The pueue handle was lost during the split session, so this evidence records the durable proof outputs produced by the run rather than the original live log.

## Proof meta excerpt

Source file:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-snix-sandbox-shell-2026-06-26/meta.json
```

Relevant fields read in-session:

```text
blocker: stage1 blocked: topology execution status was blocked
fixed_point: false
status: blocked
stage1.execution_status: blocked
stage1.failed_unit_count: 1
stage1.unit_count: 660
stage1.receipt: /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-snix-sandbox-shell-2026-06-26/stage1/receipt.json
stage1.selected_c_compiler.name: cc
stage1.selected_c_compiler.compiler_family: gcc
stage1.selected_c_compiler.content_digest_blake3: 3dccee70848def6a2838a865bceeeaffafc26a9a3249e9f7988f17cc8b14f9f2
source_built_toolchain_closure.policy_digest_blake3: c242c98d018670073a465568a5a1957e48dda771e8c793cc9b0ebc203eb01093
```

`stage1/status.txt` contains `0`, and `stage1/stderr.txt` is empty. The proof command itself completed and wrote a blocked receipt; the fixed-point proof did not complete.

## Structured receipt extraction

Steel extraction command:

```scheme
(define path (pi-arg-ref 0))
(define input (open-input-file path))
(define text (read-port-to-string input))
(close-input-port input)
(define data (pi-json-read text))
(define topology (hash-ref data 'topology_execution))
(define units (hash-ref topology 'unit_executions))
(define metadata-runs (hash-ref topology 'build_script_metadata_runs))
(define (failed-unit xs)
  (cond
    [(null? xs) #f]
    [(equal? (hash-ref (car xs) 'execution_status) "failed") (car xs)]
    [else (failed-unit (cdr xs))]))
(define failed (failed-unit units))
(define blocker (hash-ref failed 'blocker))
(define result
  (hash
    'topology_status (hash-ref topology 'execution_status)
    'unit_count (length units)
    'metadata_run_count (length metadata-runs)
    'failed_unit
      (hash
        'unit_id (hash-ref failed 'unit_id)
        'package_id (hash-ref failed 'package_id)
        'target_name (hash-ref failed 'target_name)
        'target_kind (hash-ref failed 'target_kind)
        'execution_status (hash-ref failed 'execution_status)
        'blocker (hash
          'class (hash-ref blocker 'class)
          'message (hash-ref blocker 'message)))) )
(println (pi-json-write result))
```

Output:

```json
{"failed_unit":{"blocker":{"class":"rustc-failed","message":"warning: unused import: `std::collections::BTreeSet`\n --> /home/brittonr/git/mantle/src/cargo_import.rs:1:5\n  |\n1 | use std::collections::BTreeSet;\n  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^\n  |\n  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default\n\nwarning: constant `MUSL_TARGET_GCC_ALIAS` is never used\n  --> /home/brittonr/git/mantle/src/cargo_free_self_build.rs:38:7\n   |\n38 | const MUSL_TARGET_GCC_ALIAS: &str = \"x86_64-linux-musl-gcc\";"},"execution_status":"failed","package_id":"path+native#mantle@0.1.0","target_kind":"bin","target_name":"mantle","unit_id":"native:4e60377238cb00c19fb50f30e49935e5a711ee4f71876ecf9b7f92ce579e95f0:path+native#mantle@0.1.0:mantle:bin:build"},"metadata_run_count":61,"topology_status":"blocked","unit_count":660}
```

## Result

The `SNIX_BUILD_SANDBOX_SHELL` compile-time blocker is no longer the provider fixed-point frontier: `snix-build` produced a successful native artifact in the receipt (`libsnix_build.rlib` appears in downstream dependencies), and the topology advanced to 660 executed units.

The proof remains blocked at the next deterministic frontier: the native Mantle binary unit failed under `rustc-failed` after warning diagnostics for an unused import and unused constant. This evidence makes no claim that the provider-backed fixed point succeeds.
