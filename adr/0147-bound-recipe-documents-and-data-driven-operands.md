# 0147. Bound recipe documents separately and scope data-driven operands to their map

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

Every leaf declares exactly the inputs it can address. Authoring trims GCC
specs-query and splitter leaves to the inputs their own words, runtime mounts,
cwd and guard reference. A response-driven phase additionally declares the
inputs its authenticated path map can address: each `/inputs/<name>/...`
mapping source names one input the original command emitted, so the response
can address it, while inputs with neither a word nor a mapping source are
dropped. The runtime one-to-one rule stays in force for every leaf.

## Consequences

- A stage program document up to 512 MiB parses under the recorded structural
  step bound; larger documents fail closed.
- Authoring must trim each new static leaf; an unused declaration on a static
  leaf is a build error naming the step and labels.
- The runtime invocation decoder still rejects any declared binding the decoded
  invocation does not consume; authoring must therefore scope each phase to
  its words and mapping sources.
