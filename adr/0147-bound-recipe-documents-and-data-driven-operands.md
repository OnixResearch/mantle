# 0147. Bound recipe documents separately and admit data-driven operands

Status: Accepted

## Context

A source-built proof carries each stage program as one JSON recipe document.
The Rust provider program for this stack enumerates every planned unit as a
leaf: 63 882 steps and 259 852 969 bytes. A single 64 MiB bound on every recipe
read rejected that document even though the structural step bound
(`STEP_COUNT_MAX`, 100 000) still held.

The same programs mix two operand shapes. A static leaf spells its operands as
source words, so every declared input is either read by a word, a runtime
mount source, a working directory, or a guard status input. A response-driven
leaf (an authenticated `invocation_input`, or a single `@<request>` operand)
receives its actual operands as produced data through the authenticated path
map; a particular response may not spell every bound name even though the map
can route to it.

## Decision Drivers

- The measured Rust provider program is four times the former document bound.
- Requiring one-to-one input use on data-driven leaves produced false
  rejections of correct expansions while giving no protection against dropped
  source dependencies.
- Static leaves must keep failing closed when a declaration is unused.

## Decision

Two bounds exist. `RECIPE_SOURCE_BYTES_MAX` (512 MiB) admits a stage program
document and records the measured program size; `RECIPE_BYTES_MAX` (64 MiB)
continues to bound invocation requests and manifest members.

Static leaves declare exactly the inputs they can read: authoring trims GCC
specs-query and splitter leaves to the inputs their own words, runtime mounts,
cwd and guard reference, while later phases declare their own inputs.

Response-driven leaves keep every declared input bound and are exempt from the
one-to-one use rule, because their operands are authenticated data and the path
map may address any bound name.

## Consequences

- A stage program document up to 512 MiB parses under the recorded structural
  step bound; larger documents fail closed.
- Authoring must trim each new static leaf; an unused declaration on a static
  leaf is a build error naming the step and labels.
- The runtime invocation decoder no longer rejects unused bindings on
  response-driven leaves, and the source partition check treats them as fully
  consumed.
