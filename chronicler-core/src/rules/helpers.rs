//! Helper functions for the rules engine.

use crate::dice::{self, ComponentResult, DiceExpression, DieType, RollResult};

/// Roll dice with a fallback expression. If both fail, returns a minimal result.
///
/// This avoids nested unwraps which could panic in edge cases.
pub fn roll_with_fallback(notation: &str, fallback: &str) -> RollResult {
    dice::roll(notation)
        .or_else(|_| dice::roll(fallback))
        .unwrap_or_else(|_| {
            // Create a minimal fallback result (1d4 = 1)
            let expr = DiceExpression {
                components: vec![],
                modifier: 1,
                original: fallback.to_string(),
            };
            RollResult {
                expression: expr,
                component_results: vec![ComponentResult {
                    die_type: DieType::D4,
                    rolls: vec![1],
                    kept: vec![1],
                    subtotal: 1,
                }],
                modifier: 0,
                total: 1,
                natural_20: false,
                natural_1: false,
            }
        })
}

/// Calculate the number of d6s for Sneak Attack based on Rogue level.
/// Sneak Attack scales: 1d6 at level 1, +1d6 every odd level.
pub fn sneak_attack_dice(rogue_level: u8) -> u8 {
    // 1d6 at 1, 2d6 at 3, 3d6 at 5, etc.
    rogue_level.div_ceil(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== roll_with_fallback Tests ==========

    #[test]
    fn test_roll_with_fallback_valid_notation() {
        let result = roll_with_fallback("2d6", "1d4");

        // Total should be between 2 and 12 for 2d6
        assert!(result.total >= 2 && result.total <= 12);
        assert_eq!(result.expression.original, "2d6");
    }

    #[test]
    fn test_roll_with_fallback_invalid_uses_fallback() {
        let result = roll_with_fallback("invalid", "1d6");

        // Should have used the fallback (1d6)
        assert!(result.total >= 1 && result.total <= 6);
        assert_eq!(result.expression.original, "1d6");
    }

    #[test]
    fn test_roll_with_fallback_both_invalid_returns_minimal() {
        let result = roll_with_fallback("invalid", "also_invalid");

        // Should return the minimal fallback (total of 1)
        assert_eq!(result.total, 1);
    }

    #[test]
    fn test_roll_with_fallback_with_modifier() {
        let result = roll_with_fallback("1d6+3", "1d4");

        // Total should be between 4 and 9 for 1d6+3
        assert!(result.total >= 4 && result.total <= 9);
    }

    #[test]
    fn test_roll_with_fallback_complex_notation() {
        let result = roll_with_fallback("4d6kh3", "3d6");

        // Keep highest 3 of 4d6 should be between 3 and 18
        assert!(result.total >= 3 && result.total <= 18);
    }

    // ========== sneak_attack_dice Tests ==========

    #[test]
    fn test_sneak_attack_dice_level_1() {
        assert_eq!(sneak_attack_dice(1), 1); // 1d6 at level 1
    }

    #[test]
    fn test_sneak_attack_dice_level_2() {
        assert_eq!(sneak_attack_dice(2), 1); // Still 1d6 at level 2
    }

    #[test]
    fn test_sneak_attack_dice_level_3() {
        assert_eq!(sneak_attack_dice(3), 2); // 2d6 at level 3
    }

    #[test]
    fn test_sneak_attack_dice_level_5() {
        assert_eq!(sneak_attack_dice(5), 3); // 3d6 at level 5
    }

    #[test]
    fn test_sneak_attack_dice_level_10() {
        assert_eq!(sneak_attack_dice(10), 5); // 5d6 at level 10
    }

    #[test]
    fn test_sneak_attack_dice_level_19() {
        assert_eq!(sneak_attack_dice(19), 10); // 10d6 at level 19
    }

    #[test]
    fn test_sneak_attack_dice_level_20() {
        assert_eq!(sneak_attack_dice(20), 10); // 10d6 at level 20
    }

    #[test]
    fn test_sneak_attack_dice_all_levels() {
        // Verify the formula for all levels 1-20
        let expected = [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10];
        for level in 1..=20 {
            assert_eq!(
                sneak_attack_dice(level),
                expected[level as usize - 1],
                "Failed for level {}",
                level
            );
        }
    }
}
