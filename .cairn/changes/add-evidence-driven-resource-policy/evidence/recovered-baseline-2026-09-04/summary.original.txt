# Resource-policy baseline

Date: 2026-09-04

Baseline commit: `bb5aabb2a734196123545770b4a1f1c6e61fba62`.

The existing resource lease, scheduler, action-result, store discovery, gateway,
and remote coordinator tests ran before policy-core changes. Cairn validation,
proposal gate, and design gate passed.

The broad remote-build filter reported 152 passing tests and one existing
failure in
`remote_build::tests::concurrent_resource_dispatches_commit_once_and_never_overcommit`.
The failure was a nonblocking coordinator mutation-lock acquisition that
returned `Resource temporarily unavailable`. Three exact isolated reruns
passed. This baseline records the test as a concurrency flake; it is not a
resource-policy regression.

See `commands.log`, `retry-and-lifecycle.log`, and `status.txt`.
