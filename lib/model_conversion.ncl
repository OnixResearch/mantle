# Conversion plans are admitted/lowered by mantle-model-conversion, not executed
# by evaluation. Import its JSON output, then apply this ordinary derivation
# contract. No model acquisition, ambient tools, network or special scheduler.
let Derivation = import "derivation.ncl" in
{
  derivation = fun lowered =>
    (lowered & {
      system = 'x86_64-linux,
      addressing_mode = 'input-addressed,
      sandbox = 'native,
    }) | Derivation,
}
