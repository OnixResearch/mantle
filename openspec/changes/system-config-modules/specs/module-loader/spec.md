## ADDED Requirements

### Requirement: LOADER-1 Directory scanning

The loader MUST accept a directory path and return all `*.ncl` files found at
the top level of that directory.
ID: systemconfig.module.loader.loader1

The loader MUST NOT recurse into subdirectories. If the path is missing, is not
a directory, or cannot be read, the loader MUST return a loader error naming
that path before any module evaluation begins.

#### Scenario: Loader scans only top-level module files
ID: systemconfig.module.loader.loader1.scenario

- GIVEN a module directory containing `sshd.ncl`, `nginx.ncl`, and a nested
  `subdir/firewall.ncl`
- WHEN the loader scans the directory
- THEN it returns `sshd.ncl` and `nginx.ncl`
- AND it does not return `subdir/firewall.ncl`
- AND a missing or unreadable modules directory would instead produce a loader
  error before evaluation begins

### Requirement: LOADER-2 Structural validation

Each loaded module MUST be validated against the system-module structural
contract.
ID: systemconfig.module.loader.loader2

Validation MUST confirm the presence and shape of `interface`,
`interface.roles`, and `impl`, and MUST extract async-side metadata including
all declared role names plus optional graph-driving metadata with explicit
shapes: `inputs` is an array of module-identity strings, `consumes_providers`
is an array of provider-type strings, `produces_providers` is an array of
provider-type strings, and `priority` is an integer priority value whose
default is 1000. `interface.roles` itself MUST be a record keyed by role name.
Each `interface.roles.<role>` value MUST be the Nickel contract/default value
for that role's `settings` merge: the evaluator uses that role value as the
base merged against instance settings in EVAL-3, and the loader only needs to
materialize the role name plus a handle to that role value. Modules failing
structural validation MUST produce a loader error that names the file and the
missing or malformed field.

The system-module structural contract MUST ship in crunch's embedded Nickel
stdlib as a Nickel contract, and loader validation MUST apply that contract at
the Nickel boundary before any Rust-side field extraction logic runs.

#### Scenario: Loader rejects a module with no impl function
ID: systemconfig.module.loader.loader2.scenario

- GIVEN a module record missing the `impl` field
- WHEN structural validation runs through the embedded Nickel system-module
  contract and extracts async-side role metadata
- THEN validation fails with a loader diagnostic naming `impl`
- AND the module is excluded from later evaluation

### Requirement: LOADER-3 Module identity

Each module MUST be assigned a stable identity derived from its filename stem.
ID: systemconfig.module.loader.loader3

Two modules with the same stem MUST be rejected with an error naming both file
paths. Both colliding modules MUST be excluded from the loaded-module set, but
unaffected modules MUST continue through loader output so the overall loader
behavior remains fail-open per module as required by ERR-2.

#### Scenario: Duplicate filename stems are rejected
ID: systemconfig.module.loader.loader3.scenario

- GIVEN two module files whose stems are both `nginx`
- WHEN the loader builds module identities
- THEN the loader returns an error naming both files
- AND both colliding modules are excluded while unaffected modules continue
- AND no ambiguous module identity is admitted to the pipeline

### Requirement: LOADER-4 Pure-core validation seam

The loader MUST separate filesystem discovery from handle-based validation.
ID: systemconfig.module.loader.loader4

The filesystem shell is responsible for returning `(module_name, path)` pairs.
The pure validation function MUST accept a module name, a `ValueId`, and an
`EvalThreadHandle`, and MUST return `Result<ValidatedModule, LoaderError>`
without performing filesystem I/O. Validation requests MUST refer to the
module value through `ValueId` handles rather than raw `NickelValue`
references.

If `EvalThreadHandle::evaluate_file()` fails for one module because of syntax,
import, or evaluation errors during load, that failure MUST be classified as a
loader failure for that module, the failing module MUST be excluded while
unaffected modules continue, and dependent modules MUST later receive chained
diagnostics that reference the originating load failure.

#### Scenario: Validation uses ValueId handles only
ID: systemconfig.module.loader.loader4.scenario

- GIVEN a module file already evaluated on the evaluator thread
- WHEN the async pipeline validates the module
- THEN the validation function receives the module name, `ValueId`, and
  `EvalThreadHandle`
- AND it does not read the file from disk again
- AND a different module whose `evaluate_file()` call failed would be excluded
  as a loader failure while unaffected modules continue

### Requirement: LOADER-5 Fixed limits

The loader MUST enforce a configurable maximum module count whose default value
is 1024.
ID: systemconfig.module.loader.loader5

Exceeding the limit MUST fail fast before module evaluation begins.

#### Scenario: Loader stops when the module-count limit is exceeded
ID: systemconfig.module.loader.loader5.scenario

- GIVEN a module directory containing 1025 top-level `.ncl` files
- WHEN the loader scans the directory
- THEN it fails before evaluation starts
- AND the error names the configured module-count limit
