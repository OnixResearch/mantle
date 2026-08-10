let Contracts = import "../contracts.ncl" in
let cohort = import "cohort-base.ncl" in
(cohort & { observation_method | force = "derivation-name" }) | Contracts.Cohort
