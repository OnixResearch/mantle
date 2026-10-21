let contracts = import "../contracts.ncl" in
let defaults = import "../default.ncl" in
let b = import "base.ncl" in
contracts.admit defaults.presets.stagex_inspired
  (b.policy 2 2 false [
    b.key "0000000000000000000000000000000000000000000000000000000000000001" ["source-review-approval"] b.context,
    b.key "0000000000000000000000000000000000000000000000000000000000000002" ["source-review-approval"] b.context,
  ])
