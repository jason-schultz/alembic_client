use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub class: CharacterClass,
    pub race: CharacterRace,
    pub attributes: Attributes,
    pub level: u32,
    pub experience: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Warrior,
    Mage,
    Rogue,
    Cleric,
}

impl CharacterClass {
    pub fn all() -> &'static [CharacterClass] {
        &[
            CharacterClass::Warrior,
            CharacterClass::Mage,
            CharacterClass::Rogue,
            CharacterClass::Cleric,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Mage => "Mage",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Cleric => "Cleric",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            CharacterClass::Warrior => "A mighty fighter skilled in melee combat",
            CharacterClass::Mage => "A wielder of arcane magic and powerful spells",
            CharacterClass::Rogue => "A stealthy character skilled in deception",
            CharacterClass::Cleric => "A holy warrior who heals and protects",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterRace {
    Human,
    Elf,
    Dwarf,
    Orc,
}

impl CharacterRace {
    pub fn all() -> &'static [CharacterRace] {
        &[
            CharacterRace::Human,
            CharacterRace::Elf,
            CharacterRace::Dwarf,
            CharacterRace::Orc,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            CharacterRace::Human => "Human",
            CharacterRace::Elf => "Elf",
            CharacterRace::Dwarf => "Dwarf",
            CharacterRace::Orc => "Orc",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub charisma: u32,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

impl Character {
    pub fn new(name: String, class: CharacterClass, race: CharacterRace) -> Self {
        Self {
            name,
            class,
            race,
            attributes: Attributes::default(),
            level: 1,
            experience: 0,
        }
    }
}

// Character storage/saving
pub fn save_character(character: &Character) -> Result<(), std::io::Error> {
    let save_dir = std::path::Path::new("saves");
    std::fs::create_dir_all(save_dir)?;

    let filename = format!("saves/{}.json", character.name);
    let json = serde_json::to_string_pretty(character)?;
    std::fs::write(filename, json)?;

    Ok(())
}

pub fn load_all_characters() -> Vec<Character> {
    let save_dir = std::path::Path::new("saves");
    let mut characters = Vec::new();

    if let Ok(entries) = std::fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(character) = serde_json::from_str::<Character>(&content) {
                    characters.push(character);
                }
            }
        }
    }

    characters
}

pub fn delete_character(name: &str) -> Result<(), std::io::Error> {
    let filename = format!("saves/{}.json", name);
    std::fs::remove_file(filename)?;
    Ok(())
}
