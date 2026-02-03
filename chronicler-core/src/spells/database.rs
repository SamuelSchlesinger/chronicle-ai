//! Spell database containing SRD 5.2 spell definitions.

use super::cantrips;
use super::level1;
use super::level2;
use super::level3;
use super::level4;
use super::level5;
use super::level6;
use super::level7;
use super::level8;
use super::level9;
use super::types::*;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Global spell database.
static SPELL_DATABASE: LazyLock<HashMap<String, SpellData>> = LazyLock::new(build_spell_database);

/// Look up a spell by name (case-insensitive).
pub fn get_spell(name: &str) -> Option<&'static SpellData> {
    SPELL_DATABASE.get(&name.to_lowercase())
}

/// Get all spells in the database.
pub fn all_spells() -> impl Iterator<Item = &'static SpellData> {
    SPELL_DATABASE.values()
}

/// Get all spells of a specific level.
pub fn spells_by_level(level: u8) -> impl Iterator<Item = &'static SpellData> {
    SPELL_DATABASE.values().filter(move |s| s.level == level)
}

/// Get all spells available to a class.
pub fn spells_for_class(class: SpellClass) -> impl Iterator<Item = &'static SpellData> {
    SPELL_DATABASE
        .values()
        .filter(move |s| s.classes.contains(&class))
}

fn build_spell_database() -> HashMap<String, SpellData> {
    let mut db = HashMap::new();

    // Register spells by level
    cantrips::register_cantrips(&mut db);
    level1::register_level1(&mut db);
    level2::register_level2(&mut db);
    level3::register_level3(&mut db);
    level4::register_level4(&mut db);
    level5::register_level5(&mut db);
    level6::register_level6(&mut db);
    level7::register_level7(&mut db);
    level8::register_level8(&mut db);
    level9::register_level9(&mut db);

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spell() {
        let fireball = get_spell("fireball").expect("Fireball should exist");
        assert_eq!(fireball.name, "Fireball");
        assert_eq!(fireball.level, 3);
        assert_eq!(fireball.school, SpellSchool::Evocation);
        assert!(fireball.damage_dice.is_some());
    }

    #[test]
    fn test_case_insensitive_lookup() {
        assert!(get_spell("FIREBALL").is_some());
        assert!(get_spell("Fireball").is_some());
        assert!(get_spell("fireball").is_some());
    }

    #[test]
    fn test_cantrip_scaling() {
        let fire_bolt = get_spell("fire bolt").expect("Fire Bolt should exist");
        assert!(fire_bolt.is_cantrip());

        // Level 1: 1d10
        assert_eq!(fire_bolt.cantrip_dice_count(1), 1);
        // Level 5: 2d10
        assert_eq!(fire_bolt.cantrip_dice_count(5), 2);
        // Level 11: 3d10
        assert_eq!(fire_bolt.cantrip_dice_count(11), 3);
        // Level 17: 4d10
        assert_eq!(fire_bolt.cantrip_dice_count(17), 4);
    }

    #[test]
    fn test_spell_classes() {
        let cure_wounds = get_spell("cure wounds").expect("Cure Wounds should exist");
        assert!(cure_wounds.classes.contains(&SpellClass::Cleric));
        assert!(cure_wounds.classes.contains(&SpellClass::Druid));
        assert!(!cure_wounds.classes.contains(&SpellClass::Wizard));
    }

    #[test]
    fn test_concentration_spell() {
        let hold_person = get_spell("hold person").expect("Hold Person should exist");
        assert!(hold_person.concentration);

        let fireball = get_spell("fireball").expect("Fireball should exist");
        assert!(!fireball.concentration);
    }

    #[test]
    fn test_spells_by_level() {
        let cantrips: Vec<_> = spells_by_level(0).collect();
        assert!(!cantrips.is_empty());
        for spell in cantrips {
            assert_eq!(spell.level, 0);
        }
    }

    #[test]
    fn test_spells_for_class() {
        let wizard_spells: Vec<_> = spells_for_class(SpellClass::Wizard).collect();
        assert!(!wizard_spells.is_empty());

        // Fire Bolt should be available to Wizards
        assert!(wizard_spells.iter().any(|s| s.name == "Fire Bolt"));

        // Eldritch Blast should NOT be available to Wizards
        assert!(!wizard_spells.iter().any(|s| s.name == "Eldritch Blast"));
    }
}
