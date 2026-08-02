let Contracts = import "contracts.ncl" in

{
  schema = "mantle-foreign-execution-profile-v1",
  profile_id = "mantle-foreign-nix-v1",
  scope = "foreign-bound",
  environment_mode = "declared-only",
  fixed_environment = {
    HOME = "/homeless-shelter",
    TEMP = "/build",
    TEMPDIR = "/build",
    TMP = "/build",
    TMPDIR = "/build",
  },
  protected_variables = ["HOME", "TEMP", "TEMPDIR", "TMP", "TMPDIR"],
  provide_bin_sh = true,
  work_directory = "build",
  network_mode = "deny",
  allow_setid = false,
  syscall_exceptions = [],
  writable_prefixes = ["build"],
  resource_limits = {
    min_memory_bytes = 0,
  },
  unsupported_capabilities = [],
} | Contracts.ExecutionProfile | Contracts.ExactExecutionProfile
