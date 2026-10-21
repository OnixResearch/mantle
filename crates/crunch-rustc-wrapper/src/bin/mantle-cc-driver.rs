use std::path::PathBuf;

fn main() {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let result = (|| {
        if arguments.len() < 3 || arguments[0] != "--policy" || arguments[2] != "--" {
            return Err("usage: mantle-cc-driver --policy POLICY -- COMPILER_ARGS...".to_string());
        }
        let path = PathBuf::from(&arguments[1]);
        let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() > 1_048_576 {
            return Err("cc-policy-bound".into());
        }
        let policy =
            serde_json::from_reader::<_, crunch_rustc_wrapper::cc::DriverPolicy>(file).map_err(|e| e.to_string())?;
        crunch_rustc_wrapper::cc::run_driver(&policy, &arguments[3..]).map_err(|e| e.to_string())
    })();
    match result {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            eprintln!("mantle-cc-driver:{error}");
            std::process::exit(64);
        }
    }
}
