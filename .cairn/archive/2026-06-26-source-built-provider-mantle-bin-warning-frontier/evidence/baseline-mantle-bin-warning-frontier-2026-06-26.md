# Baseline: provider-backed Mantle binary warning frontier

Task-ID: I1
Covers: r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]

## Source proof bundle

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-snix-sandbox-shell-2026-06-26
```

This is the durable provider-backed fixed-point proof output from the previous `SNIX_BUILD_SANDBOX_SHELL` change. The pueue handle was lost during the split session, so this baseline records the receipt and meta files written by the proof.

## Extraction command

```scheme
(define receipt-path (pi-arg-ref 0))
(define meta-path (pi-arg-ref 1))
(define input (open-input-file receipt-path))
(define receipt-text (read-port-to-string input))
(close-input-port input)
(define receipt (pi-json-read receipt-text))
(define topology (hash-ref receipt 'topology_execution))
(define units (hash-ref topology 'unit_executions))
(define (failed-unit xs)
  (cond
    [(null? xs) #f]
    [(equal? (hash-ref (car xs) 'execution_status) "failed") (car xs)]
    [else (failed-unit (cdr xs))]))
(define failed (failed-unit units))
(define blocker (hash-ref failed 'blocker))
(define input2 (open-input-file meta-path))
(define meta-text (read-port-to-string input2))
(close-input-port input2)
(define meta (pi-json-read meta-text))
(define stage1 (hash-ref meta 'stage1))
(define result
  (hash
   'proof_bundle (hash-ref meta 'bundle_dir)
   'fixed_point (hash-ref meta 'fixed_point)
   'status (hash-ref meta 'status)
   'stage1_status (hash-ref stage1 'execution_status)
   'stage1_unit_count (hash-ref stage1 'unit_count)
   'stage1_failed_unit_count (hash-ref stage1 'failed_unit_count)
   'topology_status (hash-ref topology 'execution_status)
   'topology_unit_count (length units)
   'failed_unit (hash
                 'unit_id (hash-ref failed 'unit_id)
                 'package_id (hash-ref failed 'package_id)
                 'target_name (hash-ref failed 'target_name)
                 'target_kind (hash-ref failed 'target_kind)
                 'execution_status (hash-ref failed 'execution_status)
                 'blocker_class (hash-ref blocker 'class)
                 'blocker_message (hash-ref blocker 'message))))
(println (pi-json-write result))
```

## Extracted baseline

```json
{"failed_unit":{"blocker_class":"rustc-failed","blocker_message":"warning: unused import: `std::collections::BTreeSet`\n --> /home/brittonr/git/mantle/src/cargo_import.rs:1:5\n  |\n1 | use std::collections::BTreeSet;\n  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^\n  |\n  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default\n\nwarning: constant `MUSL_TARGET_GCC_ALIAS` is never used\n  --> /home/brittonr/git/mantle/src/cargo_free_self_build.rs:38:7\n   |\n38 | const MUSL_TARGET_GCC_ALIAS: &str = \"x86_64-linux-musl-gcc\";","execution_status":"failed","package_id":"path+native#mantle@0.1.0","target_kind":"bin","target_name":"mantle","unit_id":"native:4e60377238cb00c19fb50f30e49935e5a711ee4f71876ecf9b7f92ce579e95f0:path+native#mantle@0.1.0:mantle:bin:build"},"fixed_point":false,"proof_bundle":"/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-snix-sandbox-shell-2026-06-26","stage1_failed_unit_count":1,"stage1_status":"blocked","stage1_unit_count":660,"status":"blocked","topology_status":"blocked","topology_unit_count":660}
```

## Baseline conclusion

The proof moved past the previous vendored `snix-build` compile-time environment blocker. The current deterministic frontier is the local native Mantle binary unit. This baseline makes no provider fixed-point success claim.
