use killian_protocol::{CharacterData, InventoryItem, Quest, Recipe};

use crate::model::{all_zones, find_zone, AppModel, ConnectField, CreationFocus, GamePanel, InputMode, NpcDef, Screen};

pub struct GatherViewProgress {
    pub action_name: String,
    pub location: String,
    pub ratio: f64,
    pub elapsed_secs: u64,
    pub total_secs: u64,
}

pub struct CombatViewProgress {
    pub mob_name: String,
    pub ratio: f64,
    pub elapsed_secs: u64,
    pub total_secs: u64,
}

#[allow(dead_code)]
pub struct ZoneView {
    pub id: &'static str,
    pub name: &'static str,
    pub region: &'static str,
    pub is_current: bool,
    pub is_reachable: bool,
}

#[allow(dead_code)]
pub struct MobView {
    pub id: &'static str,
    pub name: &'static str,
    pub level: u32,
    pub fight_duration_secs: u64,
}

#[allow(dead_code)]
pub struct GatherActionView {
    pub id: &'static str,
    pub name: &'static str,
    pub duration_secs: u64,
}

pub struct NpcView {
    pub name: &'static str,
}

pub struct CharacterCreationViewModel {
    pub race_cursor: usize,
    pub profession_cursor: usize,
    pub focus: CreationFocus,
}

pub enum AppViewModel {
    Connect(ConnectViewModel),
    CharacterCreation(CharacterCreationViewModel),
    Game(GameViewModel),
}

pub struct ConnectViewModel {
    pub nick: String,
    pub password_len: usize,
    pub server: String,
    pub notices: Vec<String>,
    pub focus: ConnectField,
}

#[allow(dead_code)]
pub struct GameViewModel {
    pub nick: String,
    pub server: String,
    pub chat_lines: Vec<String>,
    pub chat_input: String,
    pub chat_scroll: usize,
    pub character: Option<CharacterData>,
    pub inventory: Vec<InventoryItem>,
    pub inventory_cursor: usize,
    pub recipes: Vec<Recipe>,
    pub craftable: Vec<bool>,
    pub craft_cursor: usize,
    pub gather_actions: Vec<GatherActionView>,
    pub gather_cursor: usize,
    pub gathering: Option<GatherViewProgress>,
    pub zones: Vec<ZoneView>,
    pub map_cursor: &'static str,
    pub mobs: Vec<MobView>,
    pub combat_cursor: usize,
    pub combat: Option<CombatViewProgress>,
    pub game_log: Vec<String>,
    pub players_online: Vec<String>,
    pub panel_focus: GamePanel,
    pub input_mode: InputMode,
    pub map_open: bool,
    pub char_open: bool,
    pub stat_cursor: usize,
    pub npcs: Vec<NpcView>,
    pub npc_cursor: usize,
    pub equipped: Vec<String>,
    pub quests: Vec<Quest>,
}

fn client_can_craft(inventory: &[InventoryItem], recipe: &Recipe) -> bool {
    recipe.ingredients.iter().all(|ing| {
        inventory.iter().any(|item| item.name == ing.name && item.qty >= ing.qty)
    })
}

impl From<&AppModel> for AppViewModel {
    fn from(model: &AppModel) -> Self {
        match model.screen {
            Screen::Connect => AppViewModel::Connect(ConnectViewModel {
                nick: model.connect.nick.clone(),
                password_len: model.connect.password.chars().count(),
                server: model.connect.server.clone(),
                notices: model.connect.notices.clone(),
                focus: model.connect.focus,
            }),
            Screen::CharacterCreation => AppViewModel::CharacterCreation(CharacterCreationViewModel {
                race_cursor: model.creation.race_cursor,
                profession_cursor: model.creation.profession_cursor,
                focus: model.creation.focus,
            }),
            Screen::Game => {
                let gather_actions = model.gather_actions_for_zone();
                let zone_mobs = model.mobs_for_zone();
                let zone_npcs = model.npcs_for_zone();

                AppViewModel::Game(GameViewModel {
                    nick: model.connect.nick.clone(),
                    server: model.connect.server.clone(),
                    chat_lines: model.game.chat_lines.clone(),
                    chat_input: model.game.chat_input.clone(),
                    chat_scroll: model.game.chat_scroll,
                    character: model.game.character.clone(),
                    inventory: model.game.inventory.clone(),
                    inventory_cursor: model.game.inventory_cursor,
                    craftable: model.game.recipes.iter()
                        .map(|r| client_can_craft(&model.game.inventory, r))
                        .collect(),
                    recipes: model.game.recipes.clone(),
                    craft_cursor: model.game.craft_cursor,
                    gather_actions: gather_actions.iter().map(|a| GatherActionView {
                        id: a.id,
                        name: a.name,
                        duration_secs: a.duration_secs,
                    }).collect(),
                    gather_cursor: model.game.gather_cursor,
                    gathering: model.game.gathering.as_ref().map(|g| GatherViewProgress {
                        action_name: g.action_name.clone(),
                        location: g.location.clone(),
                        ratio: g.ratio(),
                        elapsed_secs: g.elapsed_secs(),
                        total_secs: g.total_secs(),
                    }),
                    zones: {
                        let current_id = model.game.current_zone;
                        let reachable = find_zone(current_id).connections;
                        all_zones().iter().map(|z| ZoneView {
                            id: z.id,
                            name: z.name,
                            region: z.region,
                            is_current: z.id == current_id,
                            is_reachable: reachable.contains(&z.id),
                        }).collect()
                    },
                    map_cursor: model.game.map_cursor,
                    mobs: zone_mobs.iter().map(|m| MobView {
                        id: m.id,
                        name: m.name,
                        level: m.level,
                        fight_duration_secs: m.fight_duration_secs,
                    }).collect(),
                    combat_cursor: model.game.combat_cursor,
                    combat: model.game.combat.as_ref().map(|c| CombatViewProgress {
                        mob_name: c.mob_name.clone(),
                        ratio: c.ratio(),
                        elapsed_secs: c.elapsed_secs(),
                        total_secs: c.total_secs(),
                    }),
                    game_log: model.game.game_log.clone(),
                    players_online: model.game.players_online.clone(),
                    panel_focus: model.game.panel_focus,
                    input_mode: model.game.input_mode,
                    map_open: model.game.map_open,
                    char_open: model.game.char_open,
                    stat_cursor: model.game.stat_cursor,
                    npcs: zone_npcs.iter().map(|n: &&NpcDef| NpcView { name: n.name }).collect(),
                    npc_cursor: model.game.npc_cursor,
                    equipped: model.game.equipped.clone(),
                    quests: model.game.quests.clone(),
                })
            }
        }
    }
}
