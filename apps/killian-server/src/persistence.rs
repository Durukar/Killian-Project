use std::path::PathBuf;

use killian_protocol::InventoryItem;

fn player_path(nick: &str) -> PathBuf {
    let dir = PathBuf::from("data/players");
    std::fs::create_dir_all(&dir).ok();
    let safe_nick = nick.replace(['/', '\\', '.', ':'], "_");
    dir.join(format!("{safe_nick}.json"))
}

pub fn load_inventory(nick: &str) -> Option<Vec<InventoryItem>> {
    let data = std::fs::read_to_string(player_path(nick)).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_inventory(nick: &str, inventory: &[InventoryItem]) {
    if let Ok(data) = serde_json::to_string(inventory) {
        std::fs::write(player_path(nick), data).ok();
    }
}
