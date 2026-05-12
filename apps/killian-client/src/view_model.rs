use killian_protocol::{CharacterData, InventoryItem, Recipe};

use crate::model::{AppModel, ConnectField, GamePanel, InputMode, Screen};

pub struct GatherViewProgress {
    pub action_name: String,
    pub location: String,
    pub ratio: f64,
    pub elapsed_secs: u64,
    pub total_secs: u64,
}

pub enum AppViewModel {
    Connect(ConnectViewModel),
    Game(GameViewModel),
}

pub struct ConnectViewModel {
    pub nick: String,
    pub password_len: usize,
    pub server: String,
    pub notices: Vec<String>,
    pub focus: ConnectField,
}

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
    pub gather_cursor: usize,
    pub gathering: Option<GatherViewProgress>,
    pub game_log: Vec<String>,
    pub players_online: Vec<String>,
    pub panel_focus: GamePanel,
    pub input_mode: InputMode,
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
            Screen::Game => AppViewModel::Game(GameViewModel {
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
                gather_cursor: model.game.gather_cursor,
                game_log: model.game.game_log.clone(),
                gathering: model.game.gathering.as_ref().map(|g| GatherViewProgress {
                    action_name: g.action_name.clone(),
                    location: g.location.clone(),
                    ratio: g.ratio(),
                    elapsed_secs: g.elapsed_secs(),
                    total_secs: g.total_secs(),
                }),
                players_online: model.game.players_online.clone(),
                panel_focus: model.game.panel_focus,
                input_mode: model.game.input_mode,
            }),
        }
    }
}
