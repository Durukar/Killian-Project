use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatLine {
    pub from: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub name: String,
    pub qty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<InventoryItem>,
    pub result: InventoryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub class_name: String,
    pub level: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub gold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Join { nick: String },
    Chat { text: String },
    Craft { recipe_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    System { text: String },
    Chat { line: ChatLine },
    CharacterUpdate { character: CharacterData },
    InventoryUpdate { items: Vec<InventoryItem> },
    RecipesUpdate { recipes: Vec<Recipe> },
    CraftResult { success: bool, message: String },
}
