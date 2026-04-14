# Tasks: Explore compiled evaluation backends

## Phase 1: Record future-work boundary

- [x] Write proposal describing where Cranelift or LLVM could fit in crunch
- [x] Write a future-work delta spec for compiled evaluation backends
- [x] Write design notes covering boundary, priorities, and non-goals

## Phase 2: Evidence before implementation

- [ ] Add or identify benchmarks that separate Nickel evaluation cost from
      build/store/sandbox cost on representative workloads
- [ ] Define success criteria that would justify evaluator codegen work
- [ ] Identify which current workloads are interpreter-bound versus I/O-bound

## Phase 3: Backend experiment plan

- [ ] Sketch a backend-neutral internal interface in `crunch-eval` for future
      experiments without changing downstream crates
- [ ] Prototype a narrow Cranelift-backed derivation-evaluation subset if the
      profiling gate is met
- [ ] Reassess LLVM only if Cranelift cannot meet measured goals or required
      optimization/target/toolchain constraints
- [ ] Define interpreter-vs-compiled equivalence tests for contracts, merges,
      package sets, recursive records, and nested derivation inputs

## Validation

- [x] Run `openspec validate explore-compiled-eval-backends`
