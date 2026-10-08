pub fn process_command_type() -> &'static str {
    core::any::type_name::<async_process::Command>()
}
