# Design

Keep the active diagnostic compact. After the existing include-flood count, compute:

- number of include-flood lines with byte length `2047`, which is the observed truncated line length from the focused make log;
- whether any include-flood line has filename-like payload (`:`, `/`, or `.h`) after the repeated diagnostic prefix.

The receipt and parity validation require `truncated_lines=2` and `filename_payload=absent`. Drift remains fail-closed.
