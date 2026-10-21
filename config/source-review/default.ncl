let contracts = import "contracts.ncl" in
{
  schema = "mantle-source-review-authoring-v1",
  default_mode = "optional",
  source_review_bytes_max = 1048576,
  review_rows_max = 64,
  trusted_keys_max = 64,
  author_keys_max = 64,
  presets = {
    reviewed = {
      mode = "required",
      required_approvals = 1,
      exclude_authors = false,
    },
    stagex_inspired = {
      mode = "required",
      required_approvals = 2,
      exclude_authors = true,
    },
  },
} | contracts.Defaults
