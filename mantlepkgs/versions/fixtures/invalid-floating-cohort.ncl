let Contracts = import "../contracts.ncl" in
let cohort = import "cohort-base.ncl" in
(cohort & {
  revisions | force = [{
    published_order = 1,
    reference = "github:NixOS/nixpkgs/nixos-unstable",
    revision = "nixos-unstable",
    nar_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  }],
}) | Contracts.Cohort
