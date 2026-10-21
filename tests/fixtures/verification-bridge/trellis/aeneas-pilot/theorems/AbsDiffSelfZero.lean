import Llbc

/-- Self-difference is zero in the generated Trellis model. -/
theorem trellis_abs_diff_self_is_zero (x : Aeneas.Std.U64) : verified_logic.arithmetic.abs_diff x x = .ok 0#u64 := by
  unfold verified_logic.arithmetic.abs_diff
  rw [if_pos (le_refl x)]
  change Aeneas.Std.UScalar.sub x x = .ok 0#u64
  simp [Aeneas.Std.UScalar.sub]
  apply Aeneas.Std.UScalar.eq_of_val_eq
  rfl
