# Probe target for the distributed-evaluation feasibility assessment.
#
# This is a deliberately static top-level record. Discovery is deterministic
# over the source text alone, so a worker that receives only the declared
# request facts must reproduce the client result byte-for-byte. Any hidden
# host-local read would surface as a response mismatch.

{
  greeting = "hello-from-client",
  message = "the client and the worker must agree",
}
