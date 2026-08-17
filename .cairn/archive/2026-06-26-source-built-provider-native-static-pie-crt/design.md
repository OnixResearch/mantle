## Context

The latest provider-backed proof bundle is:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26
```

Its stage1 receipt contains 679 unit executions and one failed unit:

```text
native:b766e54ac96cfcf5c4b1b5f6fda714924cbc7d046bb286d74e270959753b401b:path+native#mantle@0.1.0:crunch:bin:build
```

The failed derivation records `-C linker=/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26/stage1/cargo-guard-bin/cc`, so the link is already routed through the receipt-bound alias. The alias maps `-static-pie` to `-static`, but the linker still fails with source-root musl `libc.a(aio.o)` PIE relocation errors. The first-stage provider wrapper already has part of the missing shape: map `rcrt1.o` to private `crt1.o` when static-PIE is downgraded. The root native topology alias also needs to force the downgraded GCC invocation out of PIE mode with `-no-pie`, including when Rust passes the long linker argument list through a readable `@response` file.

## Decisions

### 1. Normalize CRT in the receipt-bound C compiler alias

**Choice:** Copy the manifest-declared target non-PIE CRT object into the alias runtime directory, make the generated `cc` script map `rcrt1.o` / `*/rcrt1.o` to that private `crt1.o` in direct argv and readable response-file argv, and append `-no-pie` whenever `-static-pie` is downgraded to `-static`.

**Rationale:** The root topology linker already uses the private alias. Extending that alias keeps the fix at the receipt-bound boundary and avoids changing Rust unit planning or global toolchain state. The `-no-pie` flag is scoped to observed static-PIE downgrades, so it does not alter shared or dynamic links.

### 2. Fail closed when the CRT member is missing or ambiguous

**Choice:** Require exactly one declared target CRT object (`x86_64-linux-musl-crt1.o`) for the C compiler alias normalization when a closure manifest is supplied.

**Rationale:** Guessing a CRT path from sysroot layout would reintroduce hidden fallback. The manifest is the proof surface; missing or duplicate CRT facts should block provider claims.

### 3. Keep the proof claim bounded

**Choice:** Rerun the provider-backed fixed-point proof and record the outcome as either fixed-point success or the next deterministic blocker.

**Rationale:** Local alias tests prove argument/materialization behavior only. The proof receipt remains the authority for provider frontier movement.

## Risks / Trade-offs

- The provider proof may advance to another blocker after CRT normalization. That is expected and must be recorded without claiming fixed-point success.
- The private alias script becomes more complex; tests should assert both positive mapping and negative no-ambient-Cargo behavior.
- If a future provider supplies PIE-safe CRT/libc, this non-PIE downgrade may be overly conservative. Current behavior is scoped to the source-root manifest and can be revisited with separate evidence.
