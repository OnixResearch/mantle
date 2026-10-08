pub fn host_directory() -> &'static str {
    core::any::type_name::<cap_std::fs::Dir>()
}
