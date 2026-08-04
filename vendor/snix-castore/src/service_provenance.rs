/// Generic read-through behavior for near/far service combinators.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReadThroughMode {
    /// Preserve existing cache behavior, including supported near backfill.
    #[default]
    Cache,
    /// Return far values without writing them into the near service.
    NoBackfill,
}

/// A service value with its zero-based precedence index.
///
/// Index zero is the nearest service. Each far combinator increments the index.
#[derive(Debug)]
pub struct LayeredRead<T> {
    pub value: T,
    pub layer_index: usize,
}

impl<T> LayeredRead<T> {
    #[must_use]
    pub const fn local(value: T) -> Self {
        Self { value, layer_index: 0 }
    }

    pub fn shift_far(mut self) -> Result<Self, LayerIndexOverflow> {
        self.layer_index = self.layer_index.checked_add(1).ok_or(LayerIndexOverflow)?;
        Ok(self)
    }

    #[must_use]
    pub fn map<U, F>(self, map_value: F) -> LayeredRead<U>
    where F: FnOnce(T) -> U {
        LayeredRead {
            value: map_value(self.value),
            layer_index: self.layer_index,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("read-through layer index overflow")]
pub struct LayerIndexOverflow;
