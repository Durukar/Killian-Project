use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct SavedCredentials {
    pub nick: String,
    pub password: String,
    pub server: String,
}

fn credentials_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".killian_save.json")
}

pub fn load() -> Option<SavedCredentials> {
    let data = std::fs::read_to_string(credentials_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save(nick: &str, password: &str, server: &str) {
    let creds = SavedCredentials {
        nick: nick.to_string(),
        password: password.to_string(),
        server: server.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&creds) {
        std::fs::write(credentials_path(), json).ok();
    }
}
