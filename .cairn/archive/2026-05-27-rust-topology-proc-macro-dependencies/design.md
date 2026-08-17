# Design

## Boundary

Functional core remains in `src/rust_plan.rs`: native package facts already expose normal dependencies and host targets. The implementation should derive proc-macro host dependency artifacts from those existing facts and keep execution shell changes limited to binding already-produced artifact paths before rustc launch.

## Approach

1. Extend host-unit planning so proc-macro host units use Cargo-selected normal dependency artifacts rather than every manifest edge.
2. Ensure combined topology ordering sees those host dependency edges and schedules target lib producers before the proc-macro host producer.
3. Add `--extern proc_macro` to proc-macro rustc args because direct rustc invocation does not inject it the way Cargo does.
4. Propagate selected package features into native target and host rustc args as `--cfg feature=...` so dependencies such as `syn` expose Cargo-selected modules.
5. Bind produced target dependency artifacts into proc-macro host units before execution using the existing `bind_all_dependency_artifacts` path.
6. Prove the behavior with focused tests:
   - a target derivation receives selected feature cfg args;
   - a proc-macro host derivation has `--extern proc_macro`;
   - a proc-macro host unit gets selected normal dependency artifacts such as `proc-macro2`;
   - combined topology orders dependency lib producers before the proc-macro host unit.

## Risks

- Treating all host units the same could add normal dependencies to custom build scripts. The implementation should keep normal dependency expansion scoped to proc-macro host units and leave custom-build host dependency behavior unchanged.
- Raw `proc_macro` must not be treated as a normal produced artifact; it is compiler-provided and should not enter dependency artifact digest binding.
- Manifest dependency lists can include optional or unselected edges; proc-macro host dependencies must follow captured Cargo unit graph selections, not all manifest `path_dependencies`.
