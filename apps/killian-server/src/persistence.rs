use std::path::PathBuf;

use killian_protocol::InventoryItem;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
pub struct PlayerData {
    pub password_hash: String,
    pub inventory: Vec<InventoryItem>,
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

pub fn load_inventory(nick: &str) -> Option<Vec<InventoryItem>> {
    load_player(nick).map(|p| p.inventory)
}

pub fn save_inventory(nick: &str, inventory: &[InventoryItem]) {
    if let Some(mut player) = load_player(nick) {
        player.inventory = inventory.to_vec();
        save_player(nick, &player);
    }
}
