# V3 flex 2.6.4 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.flex.2.6.4

No produced `flex-2.6.4-musl` output path is claimed in this closeout, so no runtime smoke success is claimed.

The derivation now contains fail-closed smokes that must pass once prerequisites produce a real output: `flex --version` must report `flex 2.6.4`, and installed `flex` must generate a scanner source from a tiny `.l` input.
