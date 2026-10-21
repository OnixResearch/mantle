use std::path::Path;

use crunch_verification_bridge::Producer;
use crunch_verification_bridge::consume_selected;
use crunch_verification_bridge::export;

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (handoff, destination) = if args.len() == 5 && args[0] == "octet-job" {
        (
            crunch_verification_bridge::consume_octet_job(Path::new(&args[1]), Path::new(&args[2]), &args[3])?,
            &args[4],
        )
    } else if args.len() == 6 {
        let producer = match args[0].as_str() {
            "octet" => Producer::Octet,
            "octet-verified" => Producer::OctetVerified,
            "trellis" => Producer::Trellis,
            _ => return Err("unsupported external producer".into()),
        };
        (consume_selected(producer, Path::new(&args[1]), &args[2], &args[3], &args[4])?, &args[5])
    } else {
        return Err("usage: mantle-verification-bridge <octet|octet-verified|trellis> ROOT PROFILE RECEIPT SOURCE_REVISION OUTPUT; or octet-job OWNER_OUTPUT PRODUCER_EXPORT SOURCE_REVISION OUTPUT (OUTPUT '-' inspects; Mantle does not grant theorem admission)".into());
    };
    let handoff = if destination != "-" {
        export(handoff, Path::new(destination))?
    } else {
        handoff
    };
    println!("{}", serde_json::to_string_pretty(&handoff.observation).map_err(|error| error.to_string())?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("verification bridge: {error}");
        std::process::exit(1);
    }
}
