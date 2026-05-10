use killian_protocol::{CharacterData, InventoryItem, Recipe};

use crate::model::{AppModel, ConnectField, GamePanel, Screen};

pub enum AppViewModel {
    Connect(ConnectViewModel),
    Game(GameViewModel),
}

pub struct ConnectViewModel {
    pub nick: String,
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
    pub craft_cursor: usize,
    pub panel_focus: GamePanel,
}

impl From<&AppModel> for AppViewModel {
    fn from(model: &AppModel) -> Self {
        match model.screen {
            Screen::Connect => AppViewModel::Connect(ConnectViewModel {
                nick: model.connect.nick.clone(),
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
                recipes: model.game.recipes.clone(),
                craft_cursor: model.game.craft_cursor,
                panel_focus: model.game.panel_focus,
            }),
        }
    }
}
