let contracts = import "../contracts.ncl" in
let defaults = import "../default.ncl" in
let b = import "base.ncl" in
contracts.admit defaults.presets.reviewed
  (b.policy 0 0 false [
    b.key "0000000000000000000000000000000000000000000000000000000000000001" ["source-review-approval"] b.context,
  ])
