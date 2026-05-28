## ADDED Requirements

### Requirement: Native topology honors proc-macro manifest aliases and crate types

r[rust_package_planning.native_proc_macro_crate_type_classification] Native Rust topology planning MUST classify proc-macro libraries as host proc-macro units when native manifest facts or Cargo unit crate-type facts identify them as proc macros.

#### Scenario: normalized manifest proc_macro alias is planned as host proc macro

GIVEN a native package manifest declares `[lib] proc_macro = true`
WHEN Mantle plans native package targets
THEN the library target MUST be classified as a proc-macro target
AND later topology derivation MUST build it as a host proc-macro unit instead of a target lib.

#### Scenario: lib-shaped proc-macro crate type is planned as host proc macro

GIVEN a Cargo unit target has kind `lib` or `rlib`
AND its `crate_types` contains `proc-macro`
WHEN Mantle classifies the unit for derivation and host-artifact planning
THEN the unit MUST be planned as a proc-macro host unit
AND rustc arguments MUST include `--crate-type proc-macro`
AND the produced artifact MUST be available as a host proc-macro artifact to dependent target units.

#### Scenario: ordinary lib crate types remain target libs

GIVEN a Cargo unit target has kind `lib` or `rlib`
AND its `crate_types` does not contain `proc-macro`
WHEN Mantle classifies the unit for derivation and host-artifact planning
THEN the unit MUST remain a target lib unit
AND Mantle MUST NOT synthesize a proc-macro host artifact for it.
