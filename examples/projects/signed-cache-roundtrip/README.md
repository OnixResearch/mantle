# Signed cache round trip

This project demonstrates the complete local producer → signed HTTP cache → fresh consumer path. The package is input-addressed so the consumer knows the output path before execution and can substitute it without running the builder.

The checked-in `fixtures/cache.key` is a **public test fixture private key**. Never use it outside this example. Its verifier token is:

```text
cache.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=
```

Run from this directory:

```sh
work=$(mktemp -d /tmp/mantle-signed-cache-example.XXXXXX)
mkdir -p "$work"/{producer-store,producer-state,consumer-store,consumer-state,cache}

mantle --store "$work/producer-store" --state-dir "$work/producer-state" \
  build .#payload --no-substitute --signing-key fixtures/cache.key
mantle --store "$work/producer-store" --state-dir "$work/producer-state" \
  store push --to "$work/cache" --all

busybox httpd -f -p 127.0.0.1:18081 -h "$work/cache"
# In another terminal:
mantle --store "$work/consumer-store" --state-dir "$work/consumer-state" \
  build .#payload --substituters http://127.0.0.1:18081 \
  --trusted-public-keys cache.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=
```

The workflow test checks these negative conditions:

- a missing cache entry does not cause an import
- Mantle skips an unknown signer
- wrong key material fails signature verification
- a corrupt NAR fails its declared hash check

NAR and narinfo fields use SHA-256 because the Nix cache protocol requires it. Mantle-owned action and content identities use BLAKE3.

A cache hit proves integrity and signer acceptance for this artifact. It does not prove that the producer is correct or trustworthy.
