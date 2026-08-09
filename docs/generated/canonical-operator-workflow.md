# Canonical Mantle workflow

1. Run `mantle doctor`. This command does not mutate state or use the network.
2. Run `mantle check`. This command reads project files and does not mutate them.
3. Run `mantle build --plan <root.ncl>`. This command plans realization without store mutation.
4. Run `mantle build <root.ncl>` for local realization on a supported Linux host. This step can mutate store state and use the network.
5. If local realization is unsupported, run `mantle build --builder <builder-id> --ticket-fd <fd> <root.ncl>` only for an eligible reviewed remote route. This step mutates store state and requires the network.
6. Run `mantle attest show <store-path>`. This command reads evidence and does not mutate state.
