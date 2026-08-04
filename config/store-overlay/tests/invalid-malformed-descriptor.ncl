let Contracts = import "../contracts.ncl" in
{
  allowed_state_schemas = [""],
  bind_root_inventory = true,
  bind_pathinfo = true,
  bind_directories = true,
  bind_blobs = true,
  revalidate_before_execution = true,
  revalidate_before_output_admission = true,
} | Contracts.Generation
