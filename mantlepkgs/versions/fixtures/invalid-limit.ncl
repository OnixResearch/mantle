let Contracts = import "../contracts.ncl" in
let cohort = import "cohort-base.ncl" in
(cohort & {
  limits | force = (cohort.limits & { max_revisions | force = 0 }),
}) | Contracts.Cohort
