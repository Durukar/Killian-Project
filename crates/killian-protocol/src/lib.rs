use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    #[default]
    Material,
    Weapon,
    Armor,
    Ring,
    Consumable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum StatType {
    #[default]
    Str,
    Def,
    Agi,
    Vit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatLine {
    pub from: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryItem {
    pub name: String,
    pub qty: u32,
    #[serde(default)]
    pub item_type: ItemType,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default)]
    pub str_bonus: u32,
    #[serde(default)]
    pub def_bonus: u32,
    #[serde(default)]
    pub agi_bonus: u32,
    #[serde(default)]
    pub vit_bonus: u32,
    #[serde(default)]
    pub hp_restore: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<InventoryItem>,
    pub result: InventoryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterData {
    pub class_name: String,
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub gold: u32,
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub xp_next: u32,
    #[serde(default)]
    pub str_stat: u32,
    #[serde(default)]
    pub def_stat: u32,
    #[serde(default)]
    pub agi_stat: u32,
    #[serde(default)]
    pub vit_stat: u32,
    #[serde(default)]
    pub stat_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInfo {
    pub nick: String,
    #[serde(default)]
    pub zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Join { nick: String, password: String },
    Chat { text: String },
    Craft { recipe_id: String },
    Gather { action_id: String },
    Attack { mob_id: String },
    UseItem { item_name: String },
    AllocStat { stat: StatType },
    Travel { zone_id: String },
    Equip { item_name: String },
    Unequip { item_name: String },
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
    GatherResult { message: String, items: Vec<InventoryItem> },
    CombatResult { won: bool, message: String, loot: Vec<InventoryItem> },
    JoinError { reason: String },
    PlayersUpdate { players: Vec<PlayerInfo> },
    EquipUpdate { equipped: Vec<String> },
}
