let Contracts = import "../contracts.ncl" in
let cohort = import "cohort-base.ncl" in
(cohort & { index_relative_path | force = "../escape.json" }) | Contracts.Cohort
