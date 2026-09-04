# OnixOS private remote-farm fixture

This fixture defines the producer side of one private OnixOS remote-farm endpoint.

`gateway-policy.json` uses bounded private policy. `authority.json` represents facts from a Basalt UCAN verifier.

The fixture contains no credential or private key. OnixOS supplies SecretSpec declaration metadata and systemd credential bindings during service lowering.

The fixture proves JSON contract compatibility only. It does not prove deployment, transport reachability, build correctness, KVM access, or release eligibility.
