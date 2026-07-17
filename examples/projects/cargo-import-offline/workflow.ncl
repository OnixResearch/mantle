# Review-first Cargo import workflow.
let Workflow = {
  schema | String,
  id | String,
  positive_commands | Array String,
  negative_cases | Array String,
  generated_files | Array String,
  non_claims | Array String,
} in

{
  schema = "mantle-example-workflow-v1",
  id = "cargo-import-offline",
  positive_commands = [
    "mantle --json import cargo --plan",
    "mantle --json import cargo --apply",
  ],
  negative_cases = [
    "ambiguous-default-package",
    "missing-lockfile",
    "existing-file-conflict",
  ],
  generated_files = ["mantle-project.ncl", ".mantle/inputs.ncl"],
  non_claims = [
    "not-cargo-free-execution",
    "not-network-vendoring",
    "not-full-cargo-compatibility",
  ],
} | Workflow
