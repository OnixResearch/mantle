pub const DEFAULT_NAME: &str = "world";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreetingError {
    EmptyName,
}

pub fn render(name: &str) -> Result<String, GreetingError> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(GreetingError::EmptyName);
    }

    debug_assert!(!normalized.is_empty());
    debug_assert_eq!(normalized, normalized.trim());
    Ok(format!("Hello, {normalized}!"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_normalized_name() {
        assert_eq!(render("  Mantle  ").unwrap(), "Hello, Mantle!");
        assert_eq!(render(DEFAULT_NAME).unwrap(), "Hello, world!");
    }

    #[test]
    fn rejects_an_empty_name() {
        assert_eq!(render("").unwrap_err(), GreetingError::EmptyName);
        assert_eq!(render("   ").unwrap_err(), GreetingError::EmptyName);
    }
}
