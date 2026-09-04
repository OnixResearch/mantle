pub fn forbidden() { let _ = reqwest::blocking::get("https://worker.invalid"); }
