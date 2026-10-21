import Llbc

/-- Symmetry of the generated Trellis absolute-difference model. -/
theorem trellis_abs_diff_symmetric (a b : Aeneas.Std.U64) : verified_logic.arithmetic.abs_diff a b = verified_logic.arithmetic.abs_diff b a := by
  unfold verified_logic.arithmetic.abs_diff
  by_cases hba : b ≤ a
  · by_cases hab : a ≤ b
    · have equal : a = b := le_antisymm hab hba
      subst b
      rfl
    · simp [hba, hab]
  · have hab : a ≤ b := le_of_lt (lt_of_not_ge hba)
    simp [hba, hab]
