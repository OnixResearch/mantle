let Exact = fun expected => std.contract.from_predicate (fun value =>
  std.is_string value && value == expected
) in

let SourcePin = {
  schema | Exact "mantle-nickel-export-core-source-v1",
  package | Exact "nickel-export-core",
  repository | Exact "https://github.com/OnixResearch/nickel-export",
  revision | Exact "257fafc1c746f1faf156207043a4c826bfb16d49",
  cargo | {
    source_kind | Exact "exact-git-revision",
    path_override_allowed | Bool,
    floating_ref_allowed | Bool,
  },
  nix | {
    input | Exact "nickelExportCore",
    source | Exact "github:OnixResearch/nickel-export/257fafc1c746f1faf156207043a4c826bfb16d49",
    flake | Bool,
  },
  release | {
    distribution | Exact "immutable-git-revision",
    path_override_allowed | Bool,
    floating_ref_allowed | Bool,
  },
  authority | {
    evaluator | Exact "mantle-crunch-eval",
    filesystem | Exact "mantle-shell",
    destination | Exact "mantle-shell",
    build | Exact "mantle",
    release | Exact "mantle",
  },
  non_claims | Array String,
} in

{
  schema = "mantle-nickel-export-core-source-v1",
  package = "nickel-export-core",
  repository = "https://github.com/OnixResearch/nickel-export",
  revision = "257fafc1c746f1faf156207043a4c826bfb16d49",
  cargo = {
    source_kind = "exact-git-revision",
    path_override_allowed = false,
    floating_ref_allowed = false,
  },
  nix = {
    input = "nickelExportCore",
    source = "github:OnixResearch/nickel-export/257fafc1c746f1faf156207043a4c826bfb16d49",
    flake = false,
  },
  release = {
    distribution = "immutable-git-revision",
    path_override_allowed = false,
    floating_ref_allowed = false,
  },
  authority = {
    evaluator = "mantle-crunch-eval",
    filesystem = "mantle-shell",
    destination = "mantle-shell",
    build = "mantle",
    release = "mantle",
  },
  non_claims = [
    "canonical export admission does not prove evaluator equivalence",
    "canonical export admission does not prove build correctness",
    "canonical export admission does not prove deployability",
    "canonical export admission does not prove release eligibility",
  ],
} | SourcePin
