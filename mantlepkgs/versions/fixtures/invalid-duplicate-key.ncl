let Contracts = import "../contracts.ncl" in
let cohort = import "cohort-base.ncl" in
let duplicate = {
  published_order = 1,
  reference = "github:NixOS/nixpkgs/1111111111111111111111111111111111111111",
  revision = "1111111111111111111111111111111111111111",
  nar_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
} in
(cohort & { revisions | force = [duplicate, duplicate] }) | Contracts.Cohort
