pub async fn escape() { let _ = reqwest::get("https://example.invalid").await; }
