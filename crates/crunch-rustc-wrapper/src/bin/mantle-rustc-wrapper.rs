fn main() {
    match crunch_rustc_wrapper::run_wrapper_from_environment() {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            eprintln!("mantle-rustc-wrapper:{error}");
            std::process::exit(70);
        }
    }
}
