let Responsibility = {
  name | String,
  owner | String,
  source | String,
  allowed_work | Array String,
} in

let Family = {
  name | String,
  port | String,
  adapter | String,
} in

let Architecture = {
  schema | String,
  root = {
    source | String,
    production_lines_max | Number,
    responsibilities | Array String,
  },
  responsibilities | Array Responsibility,
  families | Array Family,
  hidden_worker_exceptions | Array String,
  non_claims | Array String,
} in

{
  schema = "mantle-cli-application-architecture-v1",
  root = {
    source = "src/main.rs",
    production_lines_max = 450,
    responsibilities = [
      "CLI parse",
      "runtime context type",
      "tracing setup",
      "application entry",
      "final error presentation",
      "exit status",
    ],
  },
  responsibilities = [
    {
      name = "CLI inbound DTOs",
      owner = "cli-inbound",
      source = "src/cli_inbound.rs",
      allowed_work = ["Clap declarations", "syntax defaults", "DTO fields"],
    },
    {
      name = "application compatibility operations",
      owner = "command-family modules",
      source = "src/cli_application.rs",
      allowed_work = ["runtime context construction", "adapter selection", "legacy family delegation"],
    },
    {
      name = "typed application dispatch",
      owner = "mantle-application",
      source = "crates/mantle-application/src/application.rs",
      allowed_work = ["family port selection", "effect submission", "observation classification"],
    },
    {
      name = "dispatch policy",
      owner = "mantle-application-core",
      source = "crates/mantle-application-core/src/lib.rs",
      allowed_work = ["command admission", "effect identity", "typed outcome classification"],
    },
    {
      name = "provider and host execution",
      owner = "CLI operation adapter",
      source = "src/cli_architecture/operation_adapter.rs",
      allowed_work = ["legacy command execution", "provider error translation", "typed observation"],
    },
    {
      name = "presentation",
      owner = "CLI presentation adapter",
      source = "src/cli_architecture/presentation_adapter.rs",
      allowed_work = ["RunError mapping", "human and JSON compatibility", "reported exit status"],
    },
  ],
  families = [
    { name = "build", port = "BuildOperationPort", adapter = "CliOperationAdapter" },
    { name = "rust-plan", port = "RustPlanOperationPort", adapter = "CliOperationAdapter" },
    { name = "remote", port = "RemoteOperationPort", adapter = "CliOperationAdapter" },
    { name = "store", port = "StoreOperationPort", adapter = "CliOperationAdapter" },
    { name = "source", port = "SourceOperationPort", adapter = "CliOperationAdapter" },
    { name = "release", port = "ReleaseOperationPort", adapter = "CliOperationAdapter" },
    { name = "project", port = "ProjectOperationPort", adapter = "CliOperationAdapter" },
    { name = "bootstrap", port = "BootstrapOperationPort", adapter = "CliOperationAdapter" },
    { name = "artifact", port = "ArtifactOperationPort", adapter = "CliOperationAdapter" },
    { name = "evaluation", port = "EvaluationOperationPort", adapter = "CliOperationAdapter" },
    { name = "package", port = "PackageOperationPort", adapter = "CliOperationAdapter" },
    { name = "utility", port = "UtilityOperationPort", adapter = "CliOperationAdapter" },
  ],
  hidden_worker_exceptions = [
    "__evaluator-worker",
    "__evaluator-worker-fixture",
    "__remote-secret-worker",
  ],
  non_claims = [
    "dispatch does not prove an external effect succeeded",
    "compatibility delegation does not move provider authority into application contracts",
    "the architecture rail does not prove command semantic correctness",
    "the architecture rail does not establish release eligibility",
  ],
} | Architecture
