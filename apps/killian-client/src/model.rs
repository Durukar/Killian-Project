use std::time::{Duration, Instant};

use killian_protocol::{CharacterData, InventoryItem, Recipe, ServerMsg};

#[derive(Debug, Clone)]
pub struct GatherAction {
    pub id: &'static str,
    pub name: &'static str,
    pub location: &'static str,
    pub duration_secs: u64,
}

pub fn all_gather_actions() -> &'static [GatherAction] {
    &[
        GatherAction { id: "coletar_madeira", name: "Coletar Madeira",  location: "Floresta", duration_secs: 8  },
        GatherAction { id: "coletar_galhos",  name: "Coletar Galhos",   location: "Floresta", duration_secs: 4  },
        GatherAction { id: "minerar_pedra",   name: "Minerar Pedra",    location: "Mina",     duration_secs: 10 },
        GatherAction { id: "minerar_mineral", name: "Minerar Mineral",  location: "Mina",     duration_secs: 15 },
        GatherAction { id: "colher_ervas",    name: "Colher Ervas",     location: "Campos",   duration_secs: 5  },
    ]
}

pub struct GatherProgress {
    pub action_id: String,
    pub action_name: String,
    pub location: String,
    pub started_at: Instant,
    pub duration: Duration,
}

impl GatherProgress {
    pub fn ratio(&self) -> f64 {
        (self.started_at.elapsed().as_secs_f64() / self.duration.as_secs_f64()).min(1.0)
    }

    pub fn is_done(&self) -> bool {
        self.ratio() >= 1.0
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs().min(self.duration.as_secs())
    }

    pub fn total_secs(&self) -> u64 {
        self.duration.as_secs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Connect,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectField {
    Nick,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePanel {
    Character,
    Inventory,
    Gather,
    Craft,
    Players,
}

impl GamePanel {
    pub fn next(self) -> Self {
        match self {
            GamePanel::Character => GamePanel::Inventory,
            GamePanel::Inventory => GamePanel::Gather,
            GamePanel::Gather    => GamePanel::Craft,
            GamePanel::Craft     => GamePanel::Players,
            GamePanel::Players   => GamePanel::Character,
        }
    }
}

pub struct ConnectState {
    pub nick: String,
    pub server: String,
    pub focus: ConnectField,
    pub notices: Vec<String>,
}

pub struct GameState {
    pub chat_input: String,
    pub chat_lines: Vec<String>,
    pub chat_scroll: usize,
    pub character: Option<CharacterData>,
    pub inventory: Vec<InventoryItem>,
    pub inventory_cursor: usize,
    pub recipes: Vec<Recipe>,
    pub craft_cursor: usize,
    pub gather_cursor: usize,
    pub gathering: Option<GatherProgress>,
    pub game_log: Vec<String>,
    pub players_online: Vec<String>,
    pub panel_focus: GamePanel,
    pub input_mode: InputMode,
}

pub struct ReconnectState {
    pub nick: String,
    pub server: String,
    pub attempts: u32,
    pub next_at: Instant,
}

pub struct AppModel {
    pub screen: Screen,
    pub should_quit: bool,
    pub connecting: bool,
    pub reconnect: Option<ReconnectState>,
    pub connect: ConnectState,
    pub game: GameState,
}

impl AppModel {
    pub fn new(default_nick: String, default_server: String) -> Self {
        Self {
            screen: Screen::Connect,
            should_quit: false,
            connecting: false,
            reconnect: None,
            connect: ConnectState {
                nick: default_nick,
                server: default_server,
                focus: ConnectField::Nick,
                notices: vec![
                    "Bem-vindo ao Killian MMORPG".to_string(),
                    "Preencha nick e servidor, depois Enter para conectar.".to_string(),
                ],
            },
            game: GameState {
                chat_input: String::new(),
                chat_lines: Vec::new(),
                chat_scroll: 0,
                character: None,
                inventory: Vec::new(),
                inventory_cursor: 0,
                recipes: Vec::new(),
                craft_cursor: 0,
                gather_cursor: 0,
                gathering: None,
                game_log: Vec::new(),
                players_online: Vec::new(),
                panel_focus: GamePanel::Character,
                input_mode: InputMode::Normal,
            },
        }
    }

    pub fn toggle_connect_focus(&mut self) {
        self.connect.focus = match self.connect.focus {
            ConnectField::Nick => ConnectField::Server,
            ConnectField::Server => ConnectField::Nick,
        };
    }

    pub fn push_connect_char(&mut self, ch: char) {
        match self.connect.focus {
            ConnectField::Nick => self.connect.nick.push(ch),
            ConnectField::Server => self.connect.server.push(ch),
        }
    }

    pub fn pop_connect_char(&mut self) {
        match self.connect.focus {
            ConnectField::Nick => { self.connect.nick.pop(); }
            ConnectField::Server => { self.connect.server.pop(); }
        }
    }

    pub fn can_connect(&self) -> bool {
        !self.connect.nick.trim().is_empty() && !self.connect.server.trim().is_empty()
    }

    pub fn connect_payload(&self) -> (String, String) {
        (
            self.connect.nick.trim().to_string(),
            self.connect.server.trim().to_string(),
        )
    }

    pub fn on_ws_connected(&mut self) {
        self.connecting = true;
        self.connect.notices.push("Aguardando confirmacao do servidor...".to_string());
        self.trim_connect_notices();
    }

    pub fn on_connect_error(&mut self, message: String) {
        self.connecting = false;
        self.connect.notices.push(format!("Erro: {message}"));
        self.trim_connect_notices();
    }

    pub fn on_disconnect(&mut self) {
        self.screen = Screen::Connect;
        self.connecting = false;
        self.reconnect = None;
        self.game.character = None;
        self.game.inventory.clear();
        self.game.recipes.clear();
        self.game.players_online.clear();
        self.connect.notices.push("Desconectado. Reconecte para continuar.".to_string());
        self.trim_connect_notices();
    }

    pub fn start_reconnect(&mut self) {
        let nick = self.connect.nick.clone();
        let server = self.connect.server.clone();
        self.push_chat_system("Conexao perdida. Reconectando em 2s...".to_string());
        self.reconnect = Some(ReconnectState {
            nick,
            server,
            attempts: 0,
            next_at: Instant::now() + Duration::from_secs(2),
        });
    }

    pub fn on_reconnect_failed(&mut self) {
        self.screen = Screen::Connect;
        self.reconnect = None;
        self.game.character = None;
        self.game.inventory.clear();
        self.game.recipes.clear();
        self.game.players_online.clear();
        self.connect.notices.push("Falha ao reconectar. Tente manualmente.".to_string());
        self.trim_connect_notices();
    }

    pub fn set_panel(&mut self, panel: GamePanel) {
        self.game.panel_focus = panel;
    }

    pub fn cycle_panel_focus(&mut self) {
        self.game.panel_focus = self.game.panel_focus.next();
    }

    pub fn enter_insert_mode(&mut self) {
        self.game.input_mode = InputMode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.game.input_mode = InputMode::Normal;
    }

    pub fn cursor_up(&mut self) {
        match self.game.panel_focus {
            GamePanel::Inventory => {
                self.game.inventory_cursor = self.game.inventory_cursor.saturating_sub(1);
            }
            GamePanel::Gather => {
                self.game.gather_cursor = self.game.gather_cursor.saturating_sub(1);
            }
            GamePanel::Craft => {
                self.game.craft_cursor = self.game.craft_cursor.saturating_sub(1);
            }
            GamePanel::Character | GamePanel::Players => {
                self.game.chat_scroll = self.game.chat_scroll.saturating_add(1);
            }
        }
    }

    pub fn cursor_down(&mut self) {
        match self.game.panel_focus {
            GamePanel::Inventory => {
                let max = self.game.inventory.len().saturating_sub(1);
                self.game.inventory_cursor = (self.game.inventory_cursor + 1).min(max);
            }
            GamePanel::Gather => {
                let max = all_gather_actions().len().saturating_sub(1);
                self.game.gather_cursor = (self.game.gather_cursor + 1).min(max);
            }
            GamePanel::Craft => {
                let max = self.game.recipes.len().saturating_sub(1);
                self.game.craft_cursor = (self.game.craft_cursor + 1).min(max);
            }
            GamePanel::Character | GamePanel::Players => {
                self.game.chat_scroll = self.game.chat_scroll.saturating_sub(1);
            }
        }
    }

    pub fn selected_recipe_id(&self) -> Option<String> {
        self.game.recipes.get(self.game.craft_cursor).map(|r| r.id.clone())
    }

    pub fn start_gather(&mut self) {
        if self.game.gathering.is_some() {
            return;
        }
        let actions = all_gather_actions();
        let Some(action) = actions.get(self.game.gather_cursor) else { return };
        self.game.gathering = Some(GatherProgress {
            action_id: action.id.to_string(),
            action_name: action.name.to_string(),
            location: action.location.to_string(),
            started_at: Instant::now(),
            duration: Duration::from_secs(action.duration_secs),
        });
        self.push_chat_system(format!("Iniciando: {} ({})", action.name, action.location));
    }

    pub fn cancel_gather(&mut self) {
        if self.game.gathering.is_some() {
            self.game.gathering = None;
            self.push_chat_system("Coleta cancelada.".to_string());
        }
    }

    pub fn take_completed_gather(&mut self) -> Option<String> {
        if self.game.gathering.as_ref().map(|g| g.is_done()).unwrap_or(false) {
            let id = self.game.gathering.take().unwrap().action_id;
            Some(id)
        } else {
            None
        }
    }

    pub fn push_chat_char(&mut self, ch: char) {
        self.game.chat_input.push(ch);
    }

    pub fn pop_chat_char(&mut self) {
        self.game.chat_input.pop();
    }

    pub fn pop_chat_input_message(&mut self) -> Option<String> {
        let text = self.game.chat_input.trim().to_string();
        self.game.chat_input.clear();
        if text.is_empty() { return None; }
        Some(text)
    }

    pub fn push_server_msg(&mut self, msg: ServerMsg) {
        match msg {
            ServerMsg::System { text } => self.push_chat_system(text),
            ServerMsg::Chat { line } => {
                self.game.chat_lines.push(format!("{}: {}", line.from, line.text));
                self.trim_chat();
            }
            ServerMsg::CharacterUpdate { character } => {
                self.game.character = Some(character);
                if self.connecting {
                    self.connecting = false;
                    self.screen = Screen::Game;
                    self.game.panel_focus = GamePanel::Character;
                    self.game.input_mode = InputMode::Normal;
                    self.game.inventory_cursor = 0;
                    self.game.craft_cursor = 0;
                    self.game.gather_cursor = 0;
                    self.game.gathering = None;
                    self.game.chat_scroll = 0;
                    self.reconnect = None;
                    self.push_chat_system(format!(
                        "conectado em {} como {}",
                        self.connect.server, self.connect.nick
                    ));
                }
            }
            ServerMsg::InventoryUpdate { items } => {
                self.game.inventory = items;
                let max = self.game.inventory.len().saturating_sub(1);
                self.game.inventory_cursor = self.game.inventory_cursor.min(max);
            }
            ServerMsg::RecipesUpdate { recipes } => {
                self.game.recipes = recipes;
                let max = self.game.recipes.len().saturating_sub(1);
                self.game.craft_cursor = self.game.craft_cursor.min(max);
            }
            ServerMsg::CraftResult { success: _, message } => {
                self.push_chat_system(message);
            }
            ServerMsg::GatherResult { message, items: _ } => {
                self.push_chat_system(message);
            }
            ServerMsg::JoinError { reason } => {
                self.connecting = false;
                self.reconnect = None;
                self.screen = Screen::Connect;
                self.connect.notices.push(format!("Erro: {reason}"));
                self.trim_connect_notices();
            }
            ServerMsg::PlayersUpdate { players } => {
                self.game.players_online = players;
            }
        }
    }

    pub fn push_chat_system(&mut self, text: String) {
        self.game.game_log.push(text);
        self.trim_game_log();
    }

    fn trim_game_log(&mut self) {
        let limit = 200;
        if self.game.game_log.len() > limit {
            let excess = self.game.game_log.len() - limit;
            self.game.game_log.drain(0..excess);
        }
    }

    fn trim_chat(&mut self) {
        let limit = 500;
        if self.game.chat_lines.len() > limit {
            let excess = self.game.chat_lines.len() - limit;
            self.game.chat_lines.drain(0..excess);
        }
    }

    fn trim_connect_notices(&mut self) {
        let limit = 8;
        if self.connect.notices.len() > limit {
            let excess = self.connect.notices.len() - limit;
            self.connect.notices.drain(0..excess);
        }
    }
}
