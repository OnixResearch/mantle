# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.flex.2.5.11

- Audited `bootstrap/flex-2.5.11-musl.ncl` source pin, declared TinyCC/musl predecessor inputs, direct object compilation, link, install layout, and smoke checks.
- Intentional Crunch deviation: Flex is directly compiled from pre-generated sources with TinyCC/musl instead of relying on a stock configure/make path.
- Required failure semantics: every declared object, link, installed entrypoint, and version smoke must pass before the output contract is satisfied.
