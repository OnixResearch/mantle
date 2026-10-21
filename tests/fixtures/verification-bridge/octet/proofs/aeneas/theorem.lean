import Llbc

/-- Octet-owned compatibility theorem for an external consumer profile. -/
theorem increment_matches_model : external_consumer.increment 1#u32 = .ok 2#u32 := by
  rfl
