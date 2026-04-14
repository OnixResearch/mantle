#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoreFallbackMode {
    #[default]
    Practical,
    Strict,
}

impl StoreFallbackMode {
    pub fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }
}
