# Two independent release archive derivations with byte comparison and tamper detection.

let mantle = import "lib.ncl" in
let expected_archive_blake3 = "blake3-Roawb3qp5wK7exFYdIBJiEwtIjGz2YivY1sVtEI+YDI=" in

let source_file = fun relative_path input_name digest => mantle.fetchurl {
  url = "file://./%{relative_path}",
  hash = digest,
  name = input_name,
} in

let executable_source = source_file "src/release-demo.sh" "release-demo-script-source" "blake3-vhTFWygyvsoxSscMaFUtbGF+vKS9KpH1GJv6yGjrb7U=" in
let message_source = source_file "assets/message.txt" "release-demo-message-source" "blake3-t80MyDyDzNuhQRF2NPmvY50VKpRzCmPGlf7dRMF8IoA=" in

let release_payload = {
  name = "release-demo-payload",
  builder = "/bin/sh",
  inputs = [executable_source, message_source],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      script_input=""
      message_input=""
      for candidate in "$NIX_STORE"/*-release-demo-script-source; do
        if [ -f "$candidate" ]; then script_input="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-release-demo-message-source; do
        if [ -f "$candidate" ]; then message_input="$candidate"; break; fi
      done
      if [ -z "$script_input" ] || [ -z "$message_input" ]; then
        echo 'release payload inputs were not mounted' >&2
        exit 1
      fi
      $BB mkdir -p "$out/bin" "$out/share/release-demo"
      $BB cp "$script_input" "$out/bin/release-demo"
      $BB cp "$message_input" "$out/share/release-demo/message.txt"
      $BB chmod 0755 "$out/bin/release-demo"
      $BB chmod 0644 "$out/share/release-demo/message.txt"
    "%
  ],
} | mantle.Derivation in

let make_release = fun release_name => {
  name = release_name,
  builder = "/bin/sh",
  inputs = [release_payload],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      NORMALIZED_TIMESTAMP=197001010000
      STAGING=/tmp/release-demo-staging
      payload_input=""
      for candidate in "$NIX_STORE"/*-release-demo-payload; do
        if [ -x "$candidate/bin/release-demo" ]; then payload_input="$candidate"; break; fi
      done
      if [ -z "$payload_input" ]; then
        echo 'release payload was not mounted' >&2
        exit 1
      fi

      $BB mkdir -p "$STAGING/bin" "$STAGING/share/release-demo" "$out"
      $BB cp "$payload_input/bin/release-demo" "$STAGING/bin/release-demo"
      $BB cp "$payload_input/share/release-demo/message.txt" "$STAGING/share/release-demo/message.txt"
      $BB chmod 0755 "$STAGING/bin/release-demo"
      $BB chmod 0644 "$STAGING/share/release-demo/message.txt"
      $BB touch -t "$NORMALIZED_TIMESTAMP" "$STAGING/bin/release-demo" "$STAGING/share/release-demo/message.txt"
      cd "$STAGING"
      "$BB" tar -cf "$out/release-demo.tar" bin/release-demo share/release-demo/message.txt
      $BB printf '%s\n' '%{expected_archive_blake3}' > "$out/release-demo.tar.blake3"
    "%
  ],
} | mantle.Derivation in

let release_a = make_release "release-demo-a" in
let release_b = make_release "release-demo-b" in

let reproducibility_check = {
  name = "release-demo-reproducibility-check",
  builder = "/bin/sh",
  inputs = [release_a, release_b],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      release_a_input=""
      release_b_input=""
      for candidate in "$NIX_STORE"/*-release-demo-a; do
        if [ -f "$candidate/release-demo.tar" ]; then release_a_input="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-release-demo-b; do
        if [ -f "$candidate/release-demo.tar" ]; then release_b_input="$candidate"; break; fi
      done
      if [ -z "$release_a_input" ] || [ -z "$release_b_input" ]; then
        echo 'both release archives must be mounted' >&2
        exit 1
      fi
      "$BB" cmp "$release_a_input/release-demo.tar" "$release_b_input/release-demo.tar"
      "$BB" cmp "$release_a_input/release-demo.tar.blake3" "$release_b_input/release-demo.tar.blake3"
      "$BB" tar -tf "$release_a_input/release-demo.tar" > /tmp/archive-contents.txt
      $BB grep -x 'bin/release-demo' /tmp/archive-contents.txt >/dev/null
      $BB grep -x 'share/release-demo/message.txt' /tmp/archive-contents.txt >/dev/null
      ENTRY_COUNT_EXPECTED=2
      entry_count=$($BB wc -l < /tmp/archive-contents.txt)
      if [ "$entry_count" -ne "$ENTRY_COUNT_EXPECTED" ]; then
        echo "unexpected release archive entry count: $entry_count" >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB cp "$release_a_input/release-demo.tar.blake3" "$out/archive.blake3"
      $BB printf '%s\n' 'reproducible-release-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

let tampered_release = {
  name = "release-demo-tampered",
  builder = "/bin/sh",
  inputs = [release_b],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      release_input=""
      for candidate in "$NIX_STORE"/*-release-demo-b; do
        if [ -f "$candidate/release-demo.tar" ]; then release_input="$candidate"; break; fi
      done
      if [ -z "$release_input" ]; then
        echo 'release archive was not mounted for tampering' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB cp "$release_input/release-demo.tar" "$out/release-demo.tar"
      $BB printf '%s' 'tampered' >> "$out/release-demo.tar"
      $BB cp "$release_input/release-demo.tar.blake3" "$out/release-demo.tar.blake3"
    "%
  ],
} | mantle.Derivation in

let tamper_detection_check = {
  name = "release-demo-tamper-detection-check",
  builder = "/bin/sh",
  inputs = [release_a, tampered_release],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      release_input=""
      tampered_input=""
      for candidate in "$NIX_STORE"/*-release-demo-a; do
        if [ -f "$candidate/release-demo.tar" ]; then release_input="$candidate"; break; fi
      done
      for candidate in "$NIX_STORE"/*-release-demo-tampered; do
        if [ -f "$candidate/release-demo.tar" ]; then tampered_input="$candidate"; break; fi
      done
      if [ -z "$release_input" ] || [ -z "$tampered_input" ]; then
        echo 'release and tampered archive must be mounted' >&2
        exit 1
      fi
      if "$BB" cmp "$release_input/release-demo.tar" "$tampered_input/release-demo.tar" >/dev/null 2>&1; then
        echo 'tampered release unexpectedly matched the original archive' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'reproducible-release-tamper-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    payload = release_payload,
    release-a = release_a,
    release-b = release_b,
    tampered = tampered_release,
  },
  checks = {
    reproducible = reproducibility_check,
    tamper-detection = tamper_detection_check,
  },
  default = { package = "release-a" },
} | mantle.Project
