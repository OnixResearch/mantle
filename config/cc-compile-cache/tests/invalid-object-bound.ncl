let Contracts = import "../contracts.ncl" in
{
  max_object_bytes = 4194305,
  max_frame_bytes = 8388608,
  max_roots = 128,
  max_dependencies = 8192,
  max_arguments = 4096,
} | Contracts.LimitsPolicy
