use std::path::PathBuf;

use killian_protocol::{InventoryItem, StatType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterSave {
    pub level: u32,
    pub xp: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub gold: u32,
    pub str_stat: u32,
    pub def_stat: u32,
    pub agi_stat: u32,
    pub vit_stat: u32,
    pub stat_points: u32,
    #[serde(default)]
    pub equipped: Vec<String>,
}

fn default_character_save_inner() -> CharacterSave {
    CharacterSave {
        level: 1,
        xp: 0,
        hp: 100,
        max_hp: 100,
        gold: 150,
        str_stat: 5,
        def_stat: 5,
        agi_stat: 5,
        vit_stat: 5,
        stat_points: 0,
        equipped: Vec::new(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlayerData {
    pub password_hash: String,
    pub inventory: Vec<InventoryItem>,
    #[serde(default = "default_character_save_inner")]
    pub character: CharacterSave,
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn player_path(nick: &str) -> PathBuf {
    let dir = PathBuf::from("data/players");
    std::fs::create_dir_all(&dir).ok();
    let safe_nick = nick.replace(['/', '\\', '.', ':'], "_");
    dir.join(format!("{safe_nick}.json"))
}

pub fn load_player(nick: &str) -> Option<PlayerData> {
    let data = std::fs::read_to_string(player_path(nick)).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_player(nick: &str, data: &PlayerData) {
    if let Ok(json) = serde_json::to_string(data) {
        std::fs::write(player_path(nick), json).ok();
    }
}

pub fn default_character_save() -> CharacterSave {
    default_character_save_inner()
}

pub fn xp_for_level(level: u32) -> u32 {
    level * 100
}

pub fn check_level_up(cs: &mut CharacterSave) -> bool {
    let needed = xp_for_level(cs.level);
    if cs.xp >= needed {
        cs.xp -= needed;
        cs.level += 1;
        cs.stat_points += 3;
        cs.max_hp = 80 + (cs.vit_stat * 10) as i32;
        cs.hp = cs.max_hp;
        true
    } else {
        false
    }
}

pub fn alloc_stat(cs: &mut CharacterSave, stat: &StatType) -> bool {
    if cs.stat_points == 0 {
        return false;
    }
    cs.stat_points -= 1;
    match stat {
        StatType::Str => cs.str_stat += 1,
        StatType::Def => cs.def_stat += 1,
        StatType::Agi => cs.agi_stat += 1,
        StatType::Vit => {
            cs.vit_stat += 1;
            cs.max_hp = 80 + (cs.vit_stat * 10) as i32;
        }
    }
    true
}

pub fn save_all(nick: &str, inventory: &[InventoryItem], char_save: &CharacterSave) {
    if let Some(mut data) = load_player(nick) {
        data.inventory = inventory.to_vec();
        data.character = char_save.clone();
        save_player(nick, &data);
    }
}

