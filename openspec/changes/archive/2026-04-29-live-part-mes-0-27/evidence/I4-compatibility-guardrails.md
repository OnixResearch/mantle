Task-ID: I4
Covers: bootstrap.part.mes.0.27

# Predecessor compatibility and downstream blocker locality

Result: PASS.

Predecessor compatibility:

- Direct predecessor is `stage0-posix`.
- `bootstrap/mes.ncl` imports `stage0-posix.ncl` and declares it in `inputs = [stage0, mes_src, nyacc_src]`.
- V2 build transcript shows `stage0-posix.drv` built and resolved to:
  `target/live-part-mes-0-27/store/35ljc87nc2gcn7cxpj078qjmch8qpqzh-stage0-posix`.
- Mes build then consumed that predecessor as `STAGE0=$(find_input stage0-posix)` and completed successfully.
- Guardrail: if `stage0-posix` source/output contract changes, rerun V2/V3 here or update/reopen the `live-part-stage0-posix` predecessor change before claiming Mes remains complete.

Downstream blocker locality:

- Mes completion is scoped to building and smoking `bootstrap/mes.ncl` from `stage0-posix`.
- Downstream failures in tinycc/tcc/musl stages do not invalidate this Mes evidence unless they reveal a missing Mes output contract item.
- Separate active part changes track downstream blockers and evidence:
  - `live-part-tinycc-0-9-26`
  - `live-part-tinycc-0-9-27`
  - `live-part-tcc-musl-prep`
  - `live-part-musl-1-1-24-tcc`
  - `live-part-tcc-musl`
  - `live-part-musl-1-1-24-tcc-musl`
  - `live-part-tcc-musl-v2`

Negative space:

- This evidence does not claim downstream tcc or musl success.
- If downstream validation discovers Mes missing `mescc`, headers, M2libc, libc archives, or module paths, update this Mes part instead of hiding the defect in a downstream change.
