# crunch stdlib: FixedOutput contract.

let { HashAlgo, HashMode, .. } = import "contracts.ncl" in

{
  FixedOutput = {
    hash | String
      | doc "Expected hash in SRI format (sha256-...) or hex.",

    algo | HashAlgo
      | doc "Hash algorithm.",

    mode | HashMode
      | doc "Hashing mode: flat (single file) or recursive (NAR)."
      | default = 'flat,
  },
}
