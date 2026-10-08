pub fn network_agent_type() -> &'static str {
    core::any::type_name::<ureq::Agent>()
}
