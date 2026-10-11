use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let root = env::var("MANTLE_UNIT_APP").expect("missing admitted app root");
    let executable = format!("{root}/lib/unit_baseline_app");
    let status = Command::new(&executable).status().expect("could not execute unit binary");
    assert!(status.success(), "unit binary failed: {status}");
    fs::write(env::var("out").expect("missing consumer output"), b"cargo-unit-app-ok\n")
        .expect("could not publish consumer result");
}
