use alloc::string::String;

pub struct SelectionFacts {
    pub policy_id: String,
    pub declared_memory_bytes: u64,
    pub observed_memory_bytes: u64,
}

pub fn effective_memory_bytes(facts: SelectionFacts) -> u64 {
    facts.declared_memory_bytes.max(facts.observed_memory_bytes)
}
