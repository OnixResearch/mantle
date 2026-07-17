# Shared action-result round trip

This project keeps execution-result metadata distinct from ordinary NAR/narinfo artifact publication. The producer signs both PathInfo and its immutable `mantle-action-result-v1` record. The bounded `shared_action_result_publish` Rust helper uses Mantle's canonical action-result implementation to project the example's single local record into the static HTTP layout; it is intentionally not a general publisher.

The checked-in `fixtures/action.key` is a **public test fixture private key**. Never use it outside this example. Its verifier token is:

```text
action.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=
```

Run from this directory:

```sh
work=$(mktemp -d /tmp/mantle-action-result-example.XXXXXX)
mkdir -p "$work"/{producer-store,producer-state,consumer-store,consumer-state,cache}

mantle --json --nix-compat --store "$work/producer-store" --state-dir "$work/producer-state" \
  build .#payload --no-substitute --signing-key fixtures/action.key \
  > "$work/producer.json"
mantle --nix-compat --store "$work/producer-store" --state-dir "$work/producer-state" \
  store push --to "$work/cache" --all
cargo run -q -p mantle --example shared_action_result_publish -- \
  "$work/producer-state" "$work/cache"

busybox httpd -f -p 127.0.0.1:18082 -h "$work/cache"
# In another terminal:
mantle --json --nix-compat --store "$work/consumer-store" --state-dir "$work/consumer-state" \
  build .#payload --substituters http://127.0.0.1:18082 \
  --trusted-public-keys action.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM= \
  > "$work/consumer.json"
```

The example selects `--nix-compat` explicitly so the NAR/narinfo and action-result handoff share one interoperable logical store prefix. The consumer report must contain an `action_result_reports` row with `disposition = "reused"`, `selected_source_class = "http"`, and trusted record/PathInfo evidence. It also verifies the admitted artifact sidecar normally.

The workflow tests prove that changing either the command or the declared environment does not match the published action identity, an unknown signer is rejected and falls back to local execution, and a valid record with a missing or corrupted artifact cannot authorize reuse. Index presence is discovery only: reuse still requires record and PathInfo signatures, exact action and object identities, policy linkage, and complete admitted content. Reuse does not prove executor, source, compiler, or artifact semantics, and it is not release evidence.
