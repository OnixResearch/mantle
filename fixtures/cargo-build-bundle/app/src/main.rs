fn main() {
    println!("{}", bundle_engine::message());
    eprintln!("bundle-app: observed stderr");
    if std::env::args().any(|arg| arg == "--exit-seven") {
        std::process::exit(7);
    }
}
