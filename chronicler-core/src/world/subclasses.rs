//! D&D 5e subclasses (SRD 5.2 compatible).
//!
//! This module defines the subclass enum and associated features
//! for the one subclass per class included in the SRD 5.2.

use super::CharacterClass;
use serde::{Deserialize, Serialize};
use std::fmt;

/// D&D 5e subclasses from the SRD 5.2.
/// Each class has exactly one subclass in the SRD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Subclass {
    // Barbarian
    PathOfTheBerserker,
    // Bard
    CollegeOfLore,
    // Cleric
    LifeDomain,
    // Druid
    CircleOfTheLand,
    // Fighter
    Champion,
    // Monk
    WayOfTheOpenHand,
    // Paladin
    OathOfDevotion,
    // Ranger
    Hunter,
    // Rogue
    Thief,
    // Sorcerer
    DraconicBloodline,
    // Warlock
    TheFiend,
    // Wizard
    SchoolOfEvocation,
}

impl Subclass {
    /// Returns the display name of the subclass.
    pub fn name(&self) -> &'static str {
        match self {
            Subclass::PathOfTheBerserker => "Path of the Berserker",
            Subclass::CollegeOfLore => "College of Lore",
            Subclass::LifeDomain => "Life Domain",
            Subclass::CircleOfTheLand => "Circle of the Land",
            Subclass::Champion => "Champion",
            Subclass::WayOfTheOpenHand => "Way of the Open Hand",
            Subclass::OathOfDevotion => "Oath of Devotion",
            Subclass::Hunter => "Hunter",
            Subclass::Thief => "Thief",
            Subclass::DraconicBloodline => "Draconic Bloodline",
            Subclass::TheFiend => "The Fiend",
            Subclass::SchoolOfEvocation => "School of Evocation",
        }
    }

    /// Returns the parent class for this subclass.
    pub fn parent_class(&self) -> CharacterClass {
        match self {
            Subclass::PathOfTheBerserker => CharacterClass::Barbarian,
            Subclass::CollegeOfLore => CharacterClass::Bard,
            Subclass::LifeDomain => CharacterClass::Cleric,
            Subclass::CircleOfTheLand => CharacterClass::Druid,
            Subclass::Champion => CharacterClass::Fighter,
            Subclass::WayOfTheOpenHand => CharacterClass::Monk,
            Subclass::OathOfDevotion => CharacterClass::Paladin,
            Subclass::Hunter => CharacterClass::Ranger,
            Subclass::Thief => CharacterClass::Rogue,
            Subclass::DraconicBloodline => CharacterClass::Sorcerer,
            Subclass::TheFiend => CharacterClass::Warlock,
            Subclass::SchoolOfEvocation => CharacterClass::Wizard,
        }
    }

    /// Returns the level at which this subclass is chosen.
    pub fn subclass_level(&self) -> u8 {
        match self.parent_class() {
            CharacterClass::Cleric | CharacterClass::Sorcerer | CharacterClass::Warlock => 1,
            CharacterClass::Druid | CharacterClass::Wizard => 2,
            _ => 3, // Most classes get subclass at level 3
        }
    }

    /// Returns the levels at which this subclass grants new features.
    pub fn feature_levels(&self) -> &'static [u8] {
        match self {
            Subclass::PathOfTheBerserker => &[3, 6, 10, 14],
            Subclass::CollegeOfLore => &[3, 6, 14],
            Subclass::LifeDomain => &[1, 2, 6, 8, 17],
            Subclass::CircleOfTheLand => &[2, 3, 5, 7, 9, 10],
            Subclass::Champion => &[3, 7, 10, 15, 18],
            Subclass::WayOfTheOpenHand => &[3, 6, 11, 17],
            Subclass::OathOfDevotion => &[3, 7, 15, 20],
            Subclass::Hunter => &[3, 7, 11, 15],
            Subclass::Thief => &[3, 9, 13, 17],
            Subclass::DraconicBloodline => &[1, 6, 14, 18],
            Subclass::TheFiend => &[1, 6, 10, 14],
            Subclass::SchoolOfEvocation => &[2, 6, 10, 14],
        }
    }

    /// Returns a description of features gained at a specific level.
    pub fn features_at_level(&self, level: u8) -> Vec<SubclassFeature> {
        match self {
            Subclass::PathOfTheBerserker => match level {
                3 => vec![SubclassFeature {
                    name: "Frenzy".to_string(),
                    description: "While raging, you can make a single melee weapon attack as a bonus action on each turn. When rage ends, suffer one level of exhaustion.".to_string(),
                }],
                6 => vec![SubclassFeature {
                    name: "Mindless Rage".to_string(),
                    description: "Can't be charmed or frightened while raging. Effect suspends until rage ends.".to_string(),
                }],
                10 => vec![SubclassFeature {
                    name: "Intimidating Presence".to_string(),
                    description: "Action to frighten a creature within 30 feet (WIS save, DC 8 + prof + CHA).".to_string(),
                }],
                14 => vec![SubclassFeature {
                    name: "Retaliation".to_string(),
                    description: "When you take damage from a creature within 5 feet, use reaction to make a melee weapon attack.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::CollegeOfLore => match level {
                3 => vec![
                    SubclassFeature {
                        name: "Bonus Proficiencies".to_string(),
                        description: "Gain proficiency in three skills of your choice.".to_string(),
                    },
                    SubclassFeature {
                        name: "Cutting Words".to_string(),
                        description: "As a reaction, expend Bardic Inspiration to subtract the roll from an enemy's attack, ability check, or damage roll within 60 feet.".to_string(),
                    },
                ],
                6 => vec![SubclassFeature {
                    name: "Additional Magical Secrets".to_string(),
                    description: "Learn two spells from any class's spell list. They count as bard spells.".to_string(),
                }],
                14 => vec![SubclassFeature {
                    name: "Peerless Skill".to_string(),
                    description: "When making an ability check, expend Bardic Inspiration to add the roll to your check.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::LifeDomain => match level {
                1 => vec![
                    SubclassFeature {
                        name: "Bonus Proficiency".to_string(),
                        description: "Gain proficiency with heavy armor.".to_string(),
                    },
                    SubclassFeature {
                        name: "Disciple of Life".to_string(),
                        description: "Healing spells restore additional HP equal to 2 + the spell's level.".to_string(),
                    },
                ],
                2 => vec![SubclassFeature {
                    name: "Channel Divinity: Preserve Life".to_string(),
                    description: "Action to restore HP equal to 5 x cleric level, divided among creatures within 30 feet (max half their HP maximum each).".to_string(),
                }],
                6 => vec![SubclassFeature {
                    name: "Blessed Healer".to_string(),
                    description: "When you cast a healing spell on another creature, you also regain HP equal to 2 + the spell's level.".to_string(),
                }],
                8 => vec![SubclassFeature {
                    name: "Divine Strike".to_string(),
                    description: "Once per turn, deal an extra 1d8 radiant damage with weapon attacks. Increases to 2d8 at 14th level.".to_string(),
                }],
                17 => vec![SubclassFeature {
                    name: "Supreme Healing".to_string(),
                    description: "When rolling dice to restore HP with a healing spell, use the maximum number for each die instead.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::CircleOfTheLand => match level {
                2 => vec![
                    SubclassFeature {
                        name: "Bonus Cantrip".to_string(),
                        description: "Learn one additional druid cantrip.".to_string(),
                    },
                    SubclassFeature {
                        name: "Natural Recovery".to_string(),
                        description: "During a short rest, recover expended spell slots with combined level equal to half your druid level (rounded up). Once per long rest.".to_string(),
                    },
                ],
                3 | 5 | 7 | 9 => vec![SubclassFeature {
                    name: "Circle Spells".to_string(),
                    description: "Gain additional spells based on your chosen land type. These are always prepared and don't count against your prepared spells.".to_string(),
                }],
                10 => vec![SubclassFeature {
                    name: "Nature's Ward".to_string(),
                    description: "Can't be charmed or frightened by elementals or fey, and immune to poison and disease.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::Champion => match level {
                3 => vec![SubclassFeature {
                    name: "Improved Critical".to_string(),
                    description: "Weapon attacks score a critical hit on a roll of 19 or 20.".to_string(),
                }],
                7 => vec![SubclassFeature {
                    name: "Remarkable Athlete".to_string(),
                    description: "Add half your proficiency bonus (rounded up) to STR, DEX, and CON checks you aren't proficient in. Running long jump distance increases by your STR modifier.".to_string(),
                }],
                10 => vec![SubclassFeature {
                    name: "Additional Fighting Style".to_string(),
                    description: "Choose a second Fighting Style option.".to_string(),
                }],
                15 => vec![SubclassFeature {
                    name: "Superior Critical".to_string(),
                    description: "Weapon attacks score a critical hit on a roll of 18, 19, or 20.".to_string(),
                }],
                18 => vec![SubclassFeature {
                    name: "Survivor".to_string(),
                    description: "At the start of each turn, regain HP equal to 5 + CON modifier if you have no more than half your HP maximum and at least 1 HP.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::WayOfTheOpenHand => match level {
                3 => vec![SubclassFeature {
                    name: "Open Hand Technique".to_string(),
                    description: "When you hit with Flurry of Blows, you can impose one effect: knock prone (DEX save), push 15 feet (STR save), or prevent reactions until end of your next turn (no save).".to_string(),
                }],
                6 => vec![SubclassFeature {
                    name: "Wholeness of Body".to_string(),
                    description: "As an action, regain HP equal to 3 x monk level. Once per long rest.".to_string(),
                }],
                11 => vec![SubclassFeature {
                    name: "Tranquility".to_string(),
                    description: "At the end of a long rest, gain the effect of a Sanctuary spell (WIS save DC = 8 + WIS + prof) that lasts until your next long rest.".to_string(),
                }],
                17 => vec![SubclassFeature {
                    name: "Quivering Palm".to_string(),
                    description: "When you hit with an unarmed strike, spend 3 ki points to set up vibrations. Within days equal to your monk level, use an action to end the vibrations: CON save or drop to 0 HP, or take 10d10 necrotic damage on success.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::OathOfDevotion => match level {
                3 => vec![
                    SubclassFeature {
                        name: "Oath Spells".to_string(),
                        description: "Gain access to Protection from Evil and Good, Sanctuary (3rd), Lesser Restoration, Zone of Truth (5th), Beacon of Hope, Dispel Magic (9th), Freedom of Movement, Guardian of Faith (13th), Commune, Flame Strike (17th).".to_string(),
                    },
                    SubclassFeature {
                        name: "Channel Divinity: Sacred Weapon".to_string(),
                        description: "Action to add CHA modifier to attack rolls with one weapon for 1 minute. Weapon emits bright light 20 feet, dim 20 feet more.".to_string(),
                    },
                    SubclassFeature {
                        name: "Channel Divinity: Turn the Unholy".to_string(),
                        description: "Action to turn fiends and undead within 30 feet (WIS save or flee for 1 minute).".to_string(),
                    },
                ],
                7 => vec![SubclassFeature {
                    name: "Aura of Devotion".to_string(),
                    description: "You and friendly creatures within 10 feet can't be charmed while you're conscious. Range increases to 30 feet at 18th level.".to_string(),
                }],
                15 => vec![SubclassFeature {
                    name: "Purity of Spirit".to_string(),
                    description: "You are always under the effects of Protection from Evil and Good.".to_string(),
                }],
                20 => vec![SubclassFeature {
                    name: "Holy Nimbus".to_string(),
                    description: "Action to emanate an aura of sunlight for 1 minute. Bright light 30 feet, dim 30 feet more. Enemies starting turn in bright light take 10 radiant damage. Advantage on saves vs. fiend/undead spells. Once per long rest.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::Hunter => match level {
                3 => vec![SubclassFeature {
                    name: "Hunter's Prey".to_string(),
                    description: "Choose one: Colossus Slayer (1d8 extra damage to wounded targets), Giant Killer (reaction attack when Large+ creature misses you), or Horde Breaker (attack second creature within 5 feet of first).".to_string(),
                }],
                7 => vec![SubclassFeature {
                    name: "Defensive Tactics".to_string(),
                    description: "Choose one: Escape the Horde (opportunity attacks have disadvantage), Multiattack Defense (+4 AC after being hit by creature with multiattack), or Steel Will (advantage on frightened saves).".to_string(),
                }],
                11 => vec![SubclassFeature {
                    name: "Multiattack".to_string(),
                    description: "Choose one: Volley (ranged attack against any creatures within 10 feet of a point in range), or Whirlwind Attack (melee attack against any creatures within 5 feet).".to_string(),
                }],
                15 => vec![SubclassFeature {
                    name: "Superior Hunter's Defense".to_string(),
                    description: "Choose one: Evasion (no damage on successful DEX save, half on fail), Stand Against the Tide (redirect missed melee attack to another creature), or Uncanny Dodge (halve attack damage as reaction).".to_string(),
                }],
                _ => vec![],
            },
            Subclass::Thief => match level {
                3 => vec![
                    SubclassFeature {
                        name: "Fast Hands".to_string(),
                        description: "Use Cunning Action to make Sleight of Hand checks, use thieves' tools, or Use an Object action.".to_string(),
                    },
                    SubclassFeature {
                        name: "Second-Story Work".to_string(),
                        description: "Climbing doesn't cost extra movement. Running jump distance increases by DEX modifier feet.".to_string(),
                    },
                ],
                9 => vec![SubclassFeature {
                    name: "Supreme Sneak".to_string(),
                    description: "Advantage on Stealth checks if you move no more than half your speed on your turn.".to_string(),
                }],
                13 => vec![SubclassFeature {
                    name: "Use Magic Device".to_string(),
                    description: "Ignore all class, race, and level requirements on the use of magic items.".to_string(),
                }],
                17 => vec![SubclassFeature {
                    name: "Thief's Reflexes".to_string(),
                    description: "Take two turns during the first round of combat. First turn at normal initiative, second at initiative minus 10.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::DraconicBloodline => match level {
                1 => vec![
                    SubclassFeature {
                        name: "Dragon Ancestor".to_string(),
                        description: "Choose a dragon type. You can speak, read, and write Draconic. Double proficiency bonus on Charisma checks with dragons.".to_string(),
                    },
                    SubclassFeature {
                        name: "Draconic Resilience".to_string(),
                        description: "HP maximum increases by 1 per sorcerer level. Unarmored AC = 13 + DEX modifier.".to_string(),
                    },
                ],
                6 => vec![SubclassFeature {
                    name: "Elemental Affinity".to_string(),
                    description: "Add CHA modifier to damage rolls of spells matching your dragon's element. Spend 1 sorcery point to gain resistance to that element for 1 hour.".to_string(),
                }],
                14 => vec![SubclassFeature {
                    name: "Dragon Wings".to_string(),
                    description: "Bonus action to sprout dragon wings, gaining flying speed equal to your walking speed. Wings last until dismissed.".to_string(),
                }],
                18 => vec![SubclassFeature {
                    name: "Draconic Presence".to_string(),
                    description: "Spend 5 sorcery points to emanate an aura of awe or fear (your choice) for 1 minute. Hostile creatures within 60 feet must succeed on WIS save or be charmed (awe) or frightened (fear) until aura ends.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::TheFiend => match level {
                1 => vec![SubclassFeature {
                    name: "Dark One's Blessing".to_string(),
                    description: "When you reduce a hostile creature to 0 HP, gain temporary HP equal to CHA modifier + warlock level (minimum 1).".to_string(),
                }],
                6 => vec![SubclassFeature {
                    name: "Dark One's Own Luck".to_string(),
                    description: "When making an ability check or saving throw, add 1d10 to the roll. Once per short or long rest.".to_string(),
                }],
                10 => vec![SubclassFeature {
                    name: "Fiendish Resilience".to_string(),
                    description: "Choose a damage type at the end of a short or long rest. Gain resistance to that type until you choose another.".to_string(),
                }],
                14 => vec![SubclassFeature {
                    name: "Hurl Through Hell".to_string(),
                    description: "When you hit with an attack, send the creature through the lower planes. They disappear and take 10d10 psychic damage at the end of your next turn. Once per long rest.".to_string(),
                }],
                _ => vec![],
            },
            Subclass::SchoolOfEvocation => match level {
                2 => vec![
                    SubclassFeature {
                        name: "Evocation Savant".to_string(),
                        description: "Copying evocation spells into your spellbook costs half the normal gold and time.".to_string(),
                    },
                    SubclassFeature {
                        name: "Sculpt Spells".to_string(),
                        description: "When casting an evocation spell affecting others, choose up to 1 + spell level creatures. They automatically succeed on their saving throw and take no damage if they would normally take half.".to_string(),
                    },
                ],
                6 => vec![SubclassFeature {
                    name: "Potent Cantrip".to_string(),
                    description: "When a creature succeeds on a saving throw against your cantrip, they take half damage (if any) but no other effects.".to_string(),
                }],
                10 => vec![SubclassFeature {
                    name: "Empowered Evocation".to_string(),
                    description: "Add your INT modifier to one damage roll of any wizard evocation spell you cast.".to_string(),
                }],
                14 => vec![SubclassFeature {
                    name: "Overchannel".to_string(),
                    description: "When casting a wizard spell of 5th level or lower that deals damage, deal maximum damage. After first use between long rests, take 2d12 necrotic damage per spell level (increases with subsequent uses).".to_string(),
                }],
                _ => vec![],
            },
        }
    }

    /// Returns the available subclass for a given class.
    pub fn for_class(class: CharacterClass) -> Option<Subclass> {
        match class {
            CharacterClass::Barbarian => Some(Subclass::PathOfTheBerserker),
            CharacterClass::Bard => Some(Subclass::CollegeOfLore),
            CharacterClass::Cleric => Some(Subclass::LifeDomain),
            CharacterClass::Druid => Some(Subclass::CircleOfTheLand),
            CharacterClass::Fighter => Some(Subclass::Champion),
            CharacterClass::Monk => Some(Subclass::WayOfTheOpenHand),
            CharacterClass::Paladin => Some(Subclass::OathOfDevotion),
            CharacterClass::Ranger => Some(Subclass::Hunter),
            CharacterClass::Rogue => Some(Subclass::Thief),
            CharacterClass::Sorcerer => Some(Subclass::DraconicBloodline),
            CharacterClass::Warlock => Some(Subclass::TheFiend),
            CharacterClass::Wizard => Some(Subclass::SchoolOfEvocation),
        }
    }

    /// Returns all subclasses.
    pub fn all() -> &'static [Subclass] {
        &[
            Subclass::PathOfTheBerserker,
            Subclass::CollegeOfLore,
            Subclass::LifeDomain,
            Subclass::CircleOfTheLand,
            Subclass::Champion,
            Subclass::WayOfTheOpenHand,
            Subclass::OathOfDevotion,
            Subclass::Hunter,
            Subclass::Thief,
            Subclass::DraconicBloodline,
            Subclass::TheFiend,
            Subclass::SchoolOfEvocation,
        ]
    }
}

impl fmt::Display for Subclass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A feature granted by a subclass at a specific level.
#[derive(Debug, Clone)]
pub struct SubclassFeature {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subclass_parent_class() {
        assert_eq!(Subclass::Champion.parent_class(), CharacterClass::Fighter);
        assert_eq!(Subclass::Thief.parent_class(), CharacterClass::Rogue);
        assert_eq!(Subclass::LifeDomain.parent_class(), CharacterClass::Cleric);
    }

    #[test]
    fn test_subclass_level() {
        // Most classes get subclass at level 3
        assert_eq!(Subclass::Champion.subclass_level(), 3);
        assert_eq!(Subclass::Thief.subclass_level(), 3);

        // Clerics, Sorcerers, Warlocks get it at level 1
        assert_eq!(Subclass::LifeDomain.subclass_level(), 1);
        assert_eq!(Subclass::DraconicBloodline.subclass_level(), 1);
        assert_eq!(Subclass::TheFiend.subclass_level(), 1);

        // Druids and Wizards get it at level 2
        assert_eq!(Subclass::CircleOfTheLand.subclass_level(), 2);
        assert_eq!(Subclass::SchoolOfEvocation.subclass_level(), 2);
    }

    #[test]
    fn test_subclass_for_class() {
        assert_eq!(
            Subclass::for_class(CharacterClass::Fighter),
            Some(Subclass::Champion)
        );
        assert_eq!(
            Subclass::for_class(CharacterClass::Rogue),
            Some(Subclass::Thief)
        );
    }

    #[test]
    fn test_champion_features() {
        let features = Subclass::Champion.features_at_level(3);
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].name, "Improved Critical");

        let features = Subclass::Champion.features_at_level(15);
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].name, "Superior Critical");
    }

    #[test]
    fn test_all_subclasses() {
        let all = Subclass::all();
        assert_eq!(all.len(), 12); // One per class
    }
}
