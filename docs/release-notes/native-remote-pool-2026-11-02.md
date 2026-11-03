# Native remote build pools

`mantle build --builder-pool <id> --builder-config <nickel> --builder-ticket-fd
<endpoint=fd>` routes ready native goals through an admitted pool of transport
endpoints instead of the single-builder path. Each endpoint is selected by the
existing remote placement core, executed through a stdio transport child, and
verified with the admitted output keys before any output is exported. See
[remote build pools](../remote-pool.md) for configuration and the current
non-claims.

The pool reuses the ordinary lazy Worker, so goal dedupe, waiter notification,
the global `--jobs` bound and the session `max_concurrency` bound keep working.
A returned `mantle-plan-v1` document is scanned, its units are registered, and
their builder source inputs are transferred before the units are built.

`--remote-resource-requirements` now applies to pool goals: placement subtracts
live leases from the endpoint's declared inventory, and an endpoint without an
inventory cannot prove a quantified request. `fallback_policy = 'always` admits
a local goal only before dispatch; cancellation during that fallback reaches the
sandbox, and a completion for an attempt the coordinator no longer owns is
rejected before its outputs are exported. Each completed goal contributes one
bounded `pool_goal_observations` entry (endpoint, worker generation,
job/attempt/fence, trusted output key names); ordinary local JSON is unchanged.

The current tests exercise same-host `stdio` loopback only. `ssh-stdio` and
`p2p` are admitted by configuration but unproven, and a successful pool build
makes no release, self-hosting, distributed-determinism or physical-target
claim.
