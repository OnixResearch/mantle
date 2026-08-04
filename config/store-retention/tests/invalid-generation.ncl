let Contracts = import "../contracts.ncl" in
{
  max_roots = 1,
  max_decisions = 1,
  max_usage_objects = 1,
  max_roots_per_object = 1,
  retained_project_output_generations = 0,
  retained_project_source_generations = 1,
  max_lease_renewals = 1,
  default_lease_seconds = 1,
  max_lease_seconds = 1,
  max_closure_links_per_explanation = 1,
} | Contracts.Limits
