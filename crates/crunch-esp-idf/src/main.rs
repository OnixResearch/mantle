fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn execute() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") if args.len() == 1 => {
            let text = std::env::var("MANTLE_ESP_IDF_PLAN").map_err(|_| "missing ESP-IDF plan")?;
            let plan = serde_json::from_str(&text).map_err(|e| format!("invalid ESP-IDF plan: {e}"))?;
            let prefix = std::env::var("MANTLE_ESP_IDF_STORE_PREFIX").map_err(|_| "missing store prefix")?;
            crunch_esp_idf::run(plan, &prefix)
        }
        Some("lower") if args.len() == 5 => {
            let text = std::fs::read(&args[1]).map_err(|e| e.to_string())?;
            let plan = serde_json::from_slice(&text).map_err(|e| format!("invalid ESP-IDF plan: {e}"))?;
            let runner = crunch_esp_idf_core::Member {
                role: args[3].clone(),
                member: args[4].clone(),
            };
            let derivation = crunch_esp_idf::lower(plan, runner, &args[2])?;
            println!("{}", serde_json::to_string_pretty(&derivation).map_err(|e| e.to_string())?);
            Ok(())
        }
        _ => Err("usage: mantle-esp-idf run | lower PLAN.json STORE_PREFIX RUNNER_ROLE RUNNER_MEMBER".into()),
    }
}
