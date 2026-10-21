//! Standalone authoring helper, never a second build executor.
use std::io::Read;
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let prefix = args.next().ok_or("usage: mantle-model-conversion STORE_PREFIX < plan.json > derivation.json")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let mut input = String::new();
    std::io::stdin().take(1 << 20).read_to_string(&mut input)?;
    if input.len() >= 1 << 20 {
        return Err("plan byte bound".into());
    }
    let plan = serde_json::from_str(&input)?;
    let derivation =
        crunch_model_conversion::lower(plan, &prefix).map_err(|e| format!("model conversion rejected: {e:?}"))?;
    serde_json::to_writer_pretty(std::io::stdout().lock(), &derivation)?;
    std::io::stdout().write_all(b"\n")?;
    Ok(())
}
