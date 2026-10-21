//! Compile with the same declared static Rust toolchain as the unit helper.
//! Its signed store path is an explicit frozen consumer input, never a host helper.
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn observe(binary: &Path, out: &Path, label: &str, args: &[&str]) {
    let observed = Command::new(binary).args(args).env_clear().output().expect("execute admitted app");
    fs::write(out.join(format!("{label}.stdout")), observed.stdout).expect("write observed stdout");
    fs::write(out.join(format!("{label}.stderr")), observed.stderr).expect("write observed stderr");
    let exit = observed.status.code().expect("app terminated by signal");
    fs::write(out.join(format!("{label}.exit")), format!("{exit}\n")).expect("write observed exit");
}
fn main() {
    let root = env::var("MANTLE_UNIT_APP").expect("admitted plan-output root");
    let out = env::var("out").expect("consumer output");
    let out = Path::new(&out);
    fs::create_dir(out).expect("create observation output");
    let binary = Path::new(&root).join("lib/bundle_app");
    observe(&binary, out, "normal", &[]);
    observe(&binary, out, "nonzero", &["--exit-seven"]);
}
