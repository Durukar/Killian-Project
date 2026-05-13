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
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, PartialOrd, Ord, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Ruim,       // Cinza
    #[default]
    Common,     // Branco
    Uncommon,   // Verde
    Rare,       // Azul
    Epic,       // Roxo
    RefinedEpic, // Vermelho Roxo
    Legendary,  // Laranja (futuro - eventos mundiais)
    Broken,     // Vermelho
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobType {
    #[default]
    Common,
    Elite,
    RareElite,
    Boss,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Race {
    #[default]
    Human,
    Elf,
    Dwarf,
    Orc,
}

impl Race {
    pub fn name(self) -> &'static str {
        match self {
            Race::Human => "Humano",
            Race::Elf   => "Elfo",
            Race::Dwarf => "Anao",
            Race::Orc   => "Orc",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Race::Human => "Versatil e adaptavel. Bom em tudo, excelente em nada.",
            Race::Elf   => "Gracioso e veloz. Mestre da agilidade e percepcao.",
            Race::Dwarf => "Robusto e resiliente. Suporta golpes que matariam outros.",
            Race::Orc   => "Selvagem e brutal. Forca descomunal, defesa sacrificada.",
        }
    }

    pub fn stat_bonuses(self) -> (i32, i32, i32, i32) {
        // (str, def, agi, vit)
        match self {
            Race::Human => (2, 2, 2, 2),
            Race::Elf   => (0, 0, 5, 3),
            Race::Dwarf => (1, 3, 0, 6),
            Race::Orc   => (7, -1, 1, 2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ProfessionType {
    #[default]
    Ferreiro,
    Lenhador,
    Minerador,
    Alquimista,
}

impl ProfessionType {
    pub fn name(self) -> &'static str {
        match self {
            ProfessionType::Ferreiro   => "Ferreiro",
            ProfessionType::Lenhador   => "Lenhador",
            ProfessionType::Minerador  => "Minerador",
            ProfessionType::Alquimista => "Alquimista",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ProfessionType::Ferreiro   => "Forja armas e armaduras. Usa ferramentas de metal.",
            ProfessionType::Lenhador   => "Colhe madeira e recursos florestais. Usa machados.",
            ProfessionType::Minerador  => "Extrai minerios e pedras. Usa picaretas.",
            ProfessionType::Alquimista => "Cria pocoes e elixires. Usa caldeiroes.",
        }
    }
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
    #[serde(default)]
    pub gather_power: u32,
    #[serde(default)]
    pub craft_power: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub profession: String,
    pub min_prof_level: u32,
    pub ingredients: Vec<InventoryItem>,
    pub result: InventoryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuestObjective {
    Kill   { mob_id: String, mob_name: String, required: u32, done: u32 },
    Gather { item_name: String, required: u32, done: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub giver: String,
    pub objective: QuestObjective,
    pub reward_xp: u32,
    pub reward_gold: u32,
    pub can_turn_in: bool,
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
    #[serde(default)]
    pub race: String,
    #[serde(default)]
    pub profession: String,
    #[serde(default)]
    pub profession_level: u32,
    #[serde(default)]
    pub profession_xp: u32,
    #[serde(default)]
    pub profession_xp_next: u32,
    #[serde(default)]
    pub quests: Vec<Quest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInfo {
    pub nick: String,
    #[serde(default)]
    pub zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketListing {
    pub id: u64,
    pub seller: String,
    pub item: InventoryItem,
    pub price: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Join             { nick: String, password: String },
    CreateCharacter  { race: Race, profession: ProfessionType },
    Chat             { text: String },
    Craft            { recipe_id: String },
    Gather           { action_id: String },
    Attack           { mob_id: String },
    UseItem          { item_name: String },
    AllocStat        { stat: StatType },
    Travel           { zone_id: String },
    Equip            { item_name: String },
    Unequip          { item_name: String },
    AcceptQuest      { quest_id: String },
    TurnInQuest      { quest_id: String },
    ListItem         { item_name: String, qty: u32, price: u32 },
    BuyItem          { listing_id: u64 },
    CancelListing    { listing_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    System              { text: String },
    Chat                { line: ChatLine },
    NeedCharacterCreation,
    CharacterCreationOk,
    CharacterUpdate     { character: CharacterData },
    InventoryUpdate     { items: Vec<InventoryItem> },
    RecipesUpdate       { recipes: Vec<Recipe> },
    CraftResult         { success: bool, message: String },
    GatherResult        { message: String, items: Vec<InventoryItem> },
    CombatResult        { won: bool, message: String, loot: Vec<InventoryItem> },
    JoinError           { reason: String },
    PlayersUpdate       { players: Vec<PlayerInfo> },
    EquipUpdate         { equipped: Vec<String> },
    QuestUpdate         { quests: Vec<Quest> },
    GlobalAnnouncement  { text: String },
    MarketUpdate        { listings: Vec<MarketListing> },
    MarketResult        { success: bool, message: String },
}
