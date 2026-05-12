# Killian MMORPG POC Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adicionar personagem, inventário e craft funcional via WebSocket, com layout estilo LazyGit no cliente TUI.

**Architecture:** O servidor mantém estado por jogador (personagem + inventário) em memória local de cada conexão. O broadcast channel continua só para Chat. Mensagens de estado do jogo (CharacterUpdate, InventoryUpdate, etc.) são enviadas diretamente ao ws_writer da conexão. O cliente exibe três painéis navegáveis à esquerda e chat à direita.

**Tech Stack:** Rust, ratatui 0.30, tokio-tungstenite 0.28, serde_json, crossterm 0.29

---

## File Map

| Arquivo | Mudança |
|---------|---------|
| `crates/killian-protocol/src/lib.rs` | Adicionar ClientMsg::Craft, ServerMsg::CharacterUpdate/InventoryUpdate/RecipesUpdate/CraftResult, structs InventoryItem + Recipe |
| `apps/killian-server/src/craft.rs` | **CRIAR** — receitas hardcoded, can_craft(), apply_craft() |
| `apps/killian-server/src/main.rs` | Importar craft.rs, PlayerState por conexão, enviar estado inicial no join, tratar ClientMsg::Craft |
| `apps/killian-client/src/model.rs` | Substituir GamePanel, adicionar cursores de seleção, processar novos ServerMsg |
| `apps/killian-client/src/view_model.rs` | Adicionar campos de cursor e dados de jogo ao GameViewModel |
| `apps/killian-client/src/view.rs` | Layout LazyGit (esquerda: 3 painéis, direita: chat), cursor ▶, hints bar |
| `apps/killian-client/src/main.rs` | Teclas 1/2/3, ↑↓ com cursor, Enter craft |

---

### Task 1: Protocolo — novos tipos de mensagem

**Files:**
- Modify: `crates/killian-protocol/src/lib.rs`

- [ ] **Substituir o conteúdo completo de `lib.rs` por:**

```rust
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
```

- [ ] **Verificar que compila:**
```bash
cargo check -p killian-protocol
```
Expected: sem erros.

- [ ] **Commit:**
```bash
git add crates/killian-protocol/src/lib.rs
git commit -m "feat(protocol): add craft, inventory, character and recipe messages"
```

---

### Task 2: Servidor — craft.rs com lógica e receitas

**Files:**
- Create: `apps/killian-server/src/craft.rs`

- [ ] **Criar `apps/killian-server/src/craft.rs`:**

```rust
use killian_protocol::{InventoryItem, Recipe};

pub fn all_recipes() -> Vec<Recipe> {
    vec![
        Recipe {
            id: "pocao_media".to_string(),
            name: "Pocao Media".to_string(),
            ingredients: vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 2 }],
            result: InventoryItem { name: "Pocao Media".to_string(), qty: 1 },
        },
        Recipe {
            id: "espada_longa".to_string(),
            name: "Espada Longa".to_string(),
            ingredients: vec![
                InventoryItem { name: "Madeira".to_string(), qty: 5 },
                InventoryItem { name: "Pedra".to_string(), qty: 3 },
            ],
            result: InventoryItem { name: "Espada Longa".to_string(), qty: 1 },
        },
        Recipe {
            id: "escudo_madeira".to_string(),
            name: "Escudo de Madeira".to_string(),
            ingredients: vec![InventoryItem { name: "Madeira".to_string(), qty: 8 }],
            result: InventoryItem { name: "Escudo de Madeira".to_string(), qty: 1 },
        },
    ]
}

pub fn can_craft(inventory: &[InventoryItem], recipe: &Recipe) -> bool {
    recipe.ingredients.iter().all(|ing| {
        inventory
            .iter()
            .any(|item| item.name == ing.name && item.qty >= ing.qty)
    })
}

pub fn apply_craft(inventory: &mut Vec<InventoryItem>, recipe: &Recipe) {
    for ing in &recipe.ingredients {
        if let Some(item) = inventory.iter_mut().find(|i| i.name == ing.name) {
            item.qty -= ing.qty;
        }
    }
    inventory.retain(|i| i.qty > 0);
    if let Some(existing) = inventory.iter_mut().find(|i| i.name == recipe.result.name) {
        existing.qty += recipe.result.qty;
    } else {
        inventory.push(recipe.result.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_inventory() -> Vec<InventoryItem> {
        vec![
            InventoryItem { name: "Pocao Pequena".to_string(), qty: 3 },
            InventoryItem { name: "Madeira".to_string(), qty: 12 },
            InventoryItem { name: "Pedra".to_string(), qty: 6 },
        ]
    }

    #[test]
    fn can_craft_pocao_media_when_has_ingredients() {
        let inv = base_inventory();
        let recipe = &all_recipes()[0];
        assert!(can_craft(&inv, recipe));
    }

    #[test]
    fn cannot_craft_when_missing_ingredient() {
        let inv = vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 1 }];
        let recipe = &all_recipes()[0];
        assert!(!can_craft(&inv, recipe));
    }

    #[test]
    fn apply_craft_consumes_ingredients_and_adds_result() {
        let mut inv = base_inventory();
        let recipe = &all_recipes()[0]; // Pocao Media: 2x Pocao Pequena
        apply_craft(&mut inv, recipe);
        let pocao_p = inv.iter().find(|i| i.name == "Pocao Pequena").unwrap();
        assert_eq!(pocao_p.qty, 1);
        let pocao_m = inv.iter().find(|i| i.name == "Pocao Media").unwrap();
        assert_eq!(pocao_m.qty, 1);
    }

    #[test]
    fn apply_craft_removes_item_when_qty_reaches_zero() {
        let mut inv = vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 2 }];
        let recipe = &all_recipes()[0];
        apply_craft(&mut inv, recipe);
        assert!(!inv.iter().any(|i| i.name == "Pocao Pequena"));
    }
}
```

- [ ] **Rodar os testes:**
```bash
cargo test -p killian-server
```
Expected: 4 testes passando.

- [ ] **Commit:**
```bash
git add apps/killian-server/src/craft.rs
git commit -m "feat(server): add craft logic with recipes and tests"
```

---

### Task 3: Servidor — estado por jogador + mensagens iniciais + handle craft

**Files:**
- Modify: `apps/killian-server/src/main.rs`

- [ ] **Substituir o conteúdo completo de `main.rs`:**

```rust
mod craft;

use std::net::SocketAddr;

use craft::{all_recipes, apply_craft, can_craft};
use futures_util::{SinkExt, StreamExt};
use killian_protocol::{
    CharacterData, ChatLine, ClientMsg, InventoryItem, ServerMsg,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("KILLIAN_BIND").ok())
        .or_else(|| std::env::var("CHAT_BIND").ok())
        .unwrap_or_else(|| "0.0.0.0:7001".to_string())
        .parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("killian-server online em {}", addr);

    let (bus_tx, _bus_rx) = broadcast::channel::<ServerMsg>(512);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let bus_tx = bus_tx.clone();
        let bus_rx = bus_tx.subscribe();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, peer_addr, bus_tx, bus_rx).await {
                eprintln!("erro cliente {}: {err}", peer_addr);
            }
        });
    }
}

fn initial_inventory() -> Vec<InventoryItem> {
    vec![
        InventoryItem { name: "Pocao Pequena".to_string(), qty: 3 },
        InventoryItem { name: "Espada Curta".to_string(), qty: 1 },
        InventoryItem { name: "Madeira".to_string(), qty: 12 },
        InventoryItem { name: "Pedra".to_string(), qty: 6 },
    ]
}

fn initial_character() -> CharacterData {
    CharacterData {
        class_name: "Aventureiro".to_string(),
        level: 1,
        hp: 100,
        max_hp: 100,
        mp: 35,
        max_mp: 35,
        gold: 150,
    }
}

async fn send_msg(
    writer: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
    msg: &ServerMsg,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(msg)?;
    writer.send(Message::Text(payload.into())).await?;
    Ok(())
}

async fn handle_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    bus_tx: broadcast::Sender<ServerMsg>,
    mut bus_rx: broadcast::Receiver<ServerMsg>,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let join_line = match ws_reader.next().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        Some(Ok(_)) => return Err(anyhow::anyhow!("primeira mensagem deve ser texto JSON")),
        Some(Err(err)) => return Err(anyhow::anyhow!("erro de leitura websocket: {err}")),
        None => return Err(anyhow::anyhow!("conexao fechada antes do join")),
    };

    let nick = match serde_json::from_str::<ClientMsg>(&join_line)? {
        ClientMsg::Join { nick } => nick,
        _ => return Err(anyhow::anyhow!("primeira mensagem deve ser join")),
    };

    let mut inventory = initial_inventory();
    let character = initial_character();
    let recipes = all_recipes();

    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character }).await?;
    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::RecipesUpdate { recipes: recipes.clone() }).await?;

    let _ = bus_tx.send(ServerMsg::System {
        text: format!("{nick} entrou no jogo"),
    });

    loop {
        tokio::select! {
            incoming = ws_reader.next() => {
                let Some(incoming) = incoming else { break };

                match incoming {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientMsg>(&text) {
                            Ok(ClientMsg::Chat { text }) => {
                                let _ = bus_tx.send(ServerMsg::Chat {
                                    line: ChatLine { from: nick.clone(), text },
                                });
                            }
                            Ok(ClientMsg::Craft { recipe_id }) => {
                                let result = if let Some(recipe) = recipes.iter().find(|r| r.id == recipe_id) {
                                    if can_craft(&inventory, recipe) {
                                        apply_craft(&mut inventory, recipe);
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
                                            items: inventory.clone(),
                                        }).await?;
                                        ServerMsg::CraftResult {
                                            success: true,
                                            message: format!("{} craftado com sucesso!", recipe.name),
                                        }
                                    } else {
                                        ServerMsg::CraftResult {
                                            success: false,
                                            message: "Ingredientes insuficientes.".to_string(),
                                        }
                                    }
                                } else {
                                    ServerMsg::CraftResult {
                                        success: false,
                                        message: "Receita desconhecida.".to_string(),
                                    }
                                };
                                send_msg(&mut ws_writer, &result).await?;
                            }
                            Ok(ClientMsg::Join { .. }) => {}
                            Err(err) => {
                                eprintln!("mensagem invalida de {nick} ({peer_addr}): {err}");
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {}
                    Err(err) => {
                        let _ = bus_tx.send(ServerMsg::System {
                            text: format!("erro de websocket para {nick}: {err}"),
                        });
                        break;
                    }
                }
            }
            msg = bus_rx.recv() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        let warn = ServerMsg::System {
                            text: format!("aviso: voce perdeu {skipped} mensagens"),
                        };
                        send_msg(&mut ws_writer, &warn).await?;
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                send_msg(&mut ws_writer, &msg).await?;
            }
        }
    }

    let _ = bus_tx.send(ServerMsg::System {
        text: format!("{nick} saiu do jogo"),
    });

    Ok(())
}
```

- [ ] **Verificar que compila:**
```bash
cargo check -p killian-server
```
Expected: sem erros.

- [ ] **Commit:**
```bash
git add apps/killian-server/src/main.rs
git commit -m "feat(server): send initial state on join, handle craft messages"
```

---

### Task 4: Model do cliente — novos painéis, cursores e dados do servidor

**Files:**
- Modify: `apps/killian-client/src/model.rs`

- [ ] **Substituir o conteúdo completo de `model.rs`:**

```rust
use killian_protocol::{CharacterData, InventoryItem, Recipe, ServerMsg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Connect,
    Game,
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
    Craft,
}

impl GamePanel {
    pub fn next(self) -> Self {
        match self {
            GamePanel::Character => GamePanel::Inventory,
            GamePanel::Inventory => GamePanel::Craft,
            GamePanel::Craft => GamePanel::Character,
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
    pub panel_focus: GamePanel,
}

pub struct AppModel {
    pub screen: Screen,
    pub should_quit: bool,
    pub connect: ConnectState,
    pub game: GameState,
}

impl AppModel {
    pub fn new(default_nick: String, default_server: String) -> Self {
        Self {
            screen: Screen::Connect,
            should_quit: false,
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
                panel_focus: GamePanel::Character,
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

    pub fn on_connect_success(&mut self) {
        self.screen = Screen::Game;
        self.game.chat_scroll = 0;
        self.game.panel_focus = GamePanel::Character;
        self.game.inventory_cursor = 0;
        self.game.craft_cursor = 0;
        self.push_chat_system(format!(
            "conectado em {} como {}",
            self.connect.server, self.connect.nick
        ));
    }

    pub fn on_connect_error(&mut self, message: String) {
        self.connect.notices.push(format!("Erro: {message}"));
        self.trim_connect_notices();
    }

    pub fn on_disconnect(&mut self) {
        self.screen = Screen::Connect;
        self.game.character = None;
        self.game.inventory.clear();
        self.game.recipes.clear();
        self.connect.notices.push("Desconectado. Reconecte para continuar.".to_string());
        self.trim_connect_notices();
    }

    pub fn set_panel(&mut self, panel: GamePanel) {
        self.game.panel_focus = panel;
    }

    pub fn cycle_panel_focus(&mut self) {
        self.game.panel_focus = self.game.panel_focus.next();
    }

    pub fn cursor_up(&mut self) {
        match self.game.panel_focus {
            GamePanel::Inventory => {
                self.game.inventory_cursor = self.game.inventory_cursor.saturating_sub(1);
            }
            GamePanel::Craft => {
                self.game.craft_cursor = self.game.craft_cursor.saturating_sub(1);
            }
            GamePanel::Character => {
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
            GamePanel::Craft => {
                let max = self.game.recipes.len().saturating_sub(1);
                self.game.craft_cursor = (self.game.craft_cursor + 1).min(max);
            }
            GamePanel::Character => {
                self.game.chat_scroll = self.game.chat_scroll.saturating_sub(1);
            }
        }
    }

    pub fn selected_recipe_id(&self) -> Option<String> {
        self.game.recipes.get(self.game.craft_cursor).map(|r| r.id.clone())
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
        }
    }

    pub fn push_chat_system(&mut self, text: String) {
        self.game.chat_lines.push(format!("[sistema] {text}"));
        self.trim_chat();
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
```

- [ ] **Verificar:**
```bash
cargo check -p killian-client
```
Expected: erros nos outros módulos que ainda não foram atualizados — normal neste passo.

- [ ] **Commit:**
```bash
git add apps/killian-client/src/model.rs
git commit -m "feat(client/model): character/inventory/recipe state, cursor navigation"
```

---

### Task 5: ViewModel do cliente

**Files:**
- Modify: `apps/killian-client/src/view_model.rs`

- [ ] **Substituir o conteúdo completo de `view_model.rs`:**

```rust
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
```

- [ ] **Commit:**
```bash
git add apps/killian-client/src/view_model.rs
git commit -m "feat(client/viewmodel): expose character, inventory, recipes, cursors"
```

---

### Task 6: View do cliente — layout LazyGit

**Files:**
- Modify: `apps/killian-client/src/view.rs`

- [ ] **Substituir o conteúdo completo de `view.rs`:**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::model::{ConnectField, GamePanel};
use crate::view_model::{AppViewModel, ConnectViewModel, GameViewModel};

pub fn render(frame: &mut Frame, vm: &AppViewModel) {
    match vm {
        AppViewModel::Connect(v) => render_connect(frame, v),
        AppViewModel::Game(v) => render_game(frame, v),
    }
}

fn render_connect(frame: &mut Frame, vm: &ConnectViewModel) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(4),
        ])
        .margin(2)
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("Killian MMORPG")
            .block(Block::default().borders(Borders::ALL).title("Inicio")),
        areas[0],
    );

    let nick_style = focus_style(vm.focus == ConnectField::Nick);
    frame.render_widget(
        Paragraph::new(vm.nick.as_str()).style(nick_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(nick_style)
                .title("Nick (Tab alterna campo)"),
        ),
        areas[1],
    );

    let server_style = focus_style(vm.focus == ConnectField::Server);
    frame.render_widget(
        Paragraph::new(vm.server.as_str()).style(server_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(server_style)
                .title("Servidor WS (ex: ws://192.168.1.22:7001)"),
        ),
        areas[2],
    );

    let lines: Vec<Line<'_>> = vm.notices.iter().map(|l| Line::raw(l.as_str())).collect();
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Status (Enter conecta | Esc sai)"))
            .wrap(Wrap { trim: false }),
        areas[3],
    );

    let cursor = match vm.focus {
        ConnectField::Nick => (areas[1], vm.nick.chars().count()),
        ConnectField::Server => (areas[2], vm.server.chars().count()),
    };
    set_cursor(frame, cursor.0, cursor.1);
}

fn render_game(frame: &mut Frame, vm: &GameViewModel) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header
    let header_text = format!(
        " {} @ {}    [1] Personagem  [2] Inventario  [3] Craft",
        vm.nick, vm.server
    );
    frame.render_widget(
        Paragraph::new(header_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::BOTTOM)),
        root[0],
    );

    // Body: left panels + chat
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(0)])
        .split(root[1]);

    // Left column: 3 stacked panels
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Percentage(45),
            Constraint::Min(0),
        ])
        .split(body[0]);

    render_character_panel(frame, left[0], vm);
    render_inventory_panel(frame, left[1], vm);
    render_craft_panel(frame, left[2], vm);
    render_chat_panel(frame, body[1], vm);

    // Input bar
    let hints = "↑↓: navegar | Enter: craftar | 1-3: painel | Tab: cicla | Esc: sai";
    frame.render_widget(
        Paragraph::new(vm.chat_input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(hints)),
        root[2],
    );
    set_cursor(frame, root[2], vm.chat_input.chars().count());
}

fn render_character_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Character;
    let lines: Vec<Line<'_>> = if let Some(ch) = &vm.character {
        vec![
            Line::raw(format!(" Classe: {}", ch.class_name)),
            Line::raw(format!(" Nivel:  {}", ch.level)),
            Line::raw(format!(" HP:  {:>3}/{}", ch.hp, ch.max_hp)),
            Line::raw(format!(" MP:  {:>3}/{}", ch.mp, ch.max_mp)),
            Line::raw(format!(" Ouro: {}", ch.gold)),
        ]
    } else {
        vec![Line::raw(" Carregando...")]
    };
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(focus_style(focused))
                .title("[1] PERSONAGEM"),
        ),
        area,
    );
}

fn render_inventory_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Inventory;
    let h = area.height.saturating_sub(2) as usize;
    let total = vm.inventory.len();
    let cursor = vm.inventory_cursor;
    let start = if cursor >= h { cursor - h + 1 } else { 0 };
    let end = (start + h).min(total);

    let lines: Vec<Line<'_>> = vm.inventory[start..end]
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let idx = start + i;
            let prefix = if idx == cursor { "▶ " } else { "  " };
            let style = if idx == cursor && focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(
                format!("{}{} x{}", prefix, item.name, item.qty),
                style,
            ))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(focus_style(focused))
                .title("[2] INVENTARIO"),
        ),
        area,
    );
}

fn render_craft_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Craft;
    let h = area.height.saturating_sub(2) as usize;
    let total = vm.recipes.len();
    let cursor = vm.craft_cursor;
    let start = if cursor >= h { cursor - h + 1 } else { 0 };
    let end = (start + h).min(total);

    let lines: Vec<Line<'_>> = vm.recipes[start..end]
        .iter()
        .enumerate()
        .map(|(i, recipe)| {
            let idx = start + i;
            let prefix = if idx == cursor { "▶ " } else { "  " };
            let style = if idx == cursor && focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("{}{}", prefix, recipe.name), style))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(focus_style(focused))
                .title("[3] CRAFT (Enter: craftar)"),
        ),
        area,
    );
}

fn render_chat_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let h = area.height.saturating_sub(2) as usize;
    let total = vm.chat_lines.len();
    let offset = vm.chat_scroll.min(total.saturating_sub(h));
    let end = total.saturating_sub(offset);
    let start = end.saturating_sub(h);

    let lines: Vec<Line<'_>> = vm.chat_lines[start..end]
        .iter()
        .map(|l| Line::raw(l.as_str()))
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("CHAT"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    }
}

fn set_cursor(frame: &mut Frame, area: Rect, content_len: usize) {
    frame.set_cursor_position((area.x + 1 + content_len as u16, area.y + 1));
}
```

- [ ] **Verificar:**
```bash
cargo check -p killian-client
```
Expected: erros em `main.rs` ainda — normal.

- [ ] **Commit:**
```bash
git add apps/killian-client/src/view.rs
git commit -m "feat(client/view): LazyGit layout, cursor highlight, character/inventory/craft panels"
```

---

### Task 7: main.rs do cliente — teclas 1/2/3, cursor e craft

**Files:**
- Modify: `apps/killian-client/src/main.rs`

- [ ] **Substituir o conteúdo completo de `main.rs`:**

```rust
mod model;
mod net;
mod view;
mod view_model;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use killian_protocol::ClientMsg;
use model::{AppModel, GamePanel, Screen};
use tokio::sync::mpsc::error::TryRecvError;
use view_model::AppViewModel;

enum AppAction {
    None,
    TryConnect,
    SendChat,
    DoCraft,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let default_nick = std::env::var("KILLIAN_NICK")
        .ok()
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "adventurer".to_string());

    let default_server = std::env::var("KILLIAN_SERVER")
        .ok()
        .or_else(|| std::env::var("CHAT_SERVER").ok())
        .unwrap_or_else(|| "ws://127.0.0.1:7001".to_string());

    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, AppModel::new(default_nick, default_server)).await;
    ratatui::restore();
    result
}

async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    mut model: AppModel,
) -> anyhow::Result<()> {
    let mut net_handle: Option<net::NetHandle> = None;

    loop {
        if let Some(net) = net_handle.as_mut() {
            loop {
                match net.rx.try_recv() {
                    Ok(msg) => model.push_server_msg(msg),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        model.on_disconnect();
                        net_handle = None;
                        break;
                    }
                }
            }
        }

        let vm = AppViewModel::from(&model);
        terminal.draw(|frame| view::render(frame, &vm))?;

        if model.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            let ev = event::read()?;
            let action = handle_event(ev, &mut model);
            process_action(action, &mut model, &mut net_handle).await;
        }
    }

    Ok(())
}

fn handle_event(ev: Event, model: &mut AppModel) -> AppAction {
    let Event::Key(key) = ev else { return AppAction::None };
    if key.kind != KeyEventKind::Press { return AppAction::None }

    match model.screen {
        Screen::Connect => handle_connect_key(key, model),
        Screen::Game => handle_game_key(key, model),
    }
}

fn handle_connect_key(key: crossterm::event::KeyEvent, model: &mut AppModel) -> AppAction {
    match key.code {
        KeyCode::Esc => { model.should_quit = true; AppAction::None }
        KeyCode::Tab | KeyCode::BackTab => { model.toggle_connect_focus(); AppAction::None }
        KeyCode::Enter => AppAction::TryConnect,
        KeyCode::Backspace => { model.pop_connect_char(); AppAction::None }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            model.should_quit = true; AppAction::None
        }
        KeyCode::Char(ch) => { model.push_connect_char(ch); AppAction::None }
        _ => AppAction::None,
    }
}

fn handle_game_key(key: crossterm::event::KeyEvent, model: &mut AppModel) -> AppAction {
    match key.code {
        KeyCode::Esc => { model.should_quit = true; AppAction::None }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            model.should_quit = true; AppAction::None
        }
        KeyCode::Char('1') => { model.set_panel(GamePanel::Character); AppAction::None }
        KeyCode::Char('2') => { model.set_panel(GamePanel::Inventory); AppAction::None }
        KeyCode::Char('3') => { model.set_panel(GamePanel::Craft); AppAction::None }
        KeyCode::Tab => { model.cycle_panel_focus(); AppAction::None }
        KeyCode::Up => { model.cursor_up(); AppAction::None }
        KeyCode::Down => { model.cursor_down(); AppAction::None }
        KeyCode::Enter => {
            if model.game.panel_focus == GamePanel::Craft {
                AppAction::DoCraft
            } else {
                AppAction::SendChat
            }
        }
        KeyCode::Backspace => { model.pop_chat_char(); AppAction::None }
        KeyCode::Char(ch) => { model.push_chat_char(ch); AppAction::None }
        _ => AppAction::None,
    }
}

async fn process_action(
    action: AppAction,
    model: &mut AppModel,
    net_handle: &mut Option<net::NetHandle>,
) {
    match action {
        AppAction::None => {}
        AppAction::TryConnect => {
            if !model.can_connect() {
                model.on_connect_error("Nick e servidor sao obrigatorios.".to_string());
                return;
            }
            let (nick, server) = model.connect_payload();
            model.connect.notices.push(format!("Conectando em {server} ..."));
            match net::connect(&server, nick).await {
                Ok(handle) => { *net_handle = Some(handle); model.on_connect_success(); }
                Err(err) => { model.on_connect_error(err.to_string()); }
            }
        }
        AppAction::SendChat => {
            let Some(text) = model.pop_chat_input_message() else { return };
            let Some(net) = net_handle.as_ref() else {
                model.push_chat_system("sem conexao ativa".to_string());
                return;
            };
            if net.tx.send(ClientMsg::Chat { text }).is_err() {
                model.push_chat_system("falha ao enviar mensagem".to_string());
            }
        }
        AppAction::DoCraft => {
            let Some(recipe_id) = model.selected_recipe_id() else { return };
            let Some(net) = net_handle.as_ref() else {
                model.push_chat_system("sem conexao ativa".to_string());
                return;
            };
            if net.tx.send(ClientMsg::Craft { recipe_id }).is_err() {
                model.push_chat_system("falha ao enviar craft".to_string());
            }
        }
    }
}
```

- [ ] **Build final:**
```bash
cargo build --workspace
```
Expected: `Finished` sem erros nem warnings relevantes.

- [ ] **Rodar todos os testes:**
```bash
cargo test --workspace
```
Expected: 4 testes passando (craft.rs).

- [ ] **Commit:**
```bash
git add apps/killian-client/src/main.rs
git commit -m "feat(client): 1/2/3 panel jump, arrow cursor, enter craft action"
```

---

### Task 8: Teste manual end-to-end

- [ ] **Terminal 1 — servidor:**
```bash
cargo run -p killian-server
```
Expected: `killian-server online em 0.0.0.0:7001`

- [ ] **Terminal 2 — cliente:**
```bash
cargo run -p killian-client
```

- [ ] **Verificar tela de conexão:**
  - Campos Nick e Server visíveis
  - Tab alterna foco (borda amarela)
  - Preenche nick, deixa server como `ws://127.0.0.1:7001`, pressiona Enter

- [ ] **Verificar game screen:**
  - Três painéis à esquerda: PERSONAGEM, INVENTARIO, CRAFT
  - CHAT ocupa a direita
  - Painel PERSONAGEM mostra dados (Classe, Nivel, HP, MP, Ouro)
  - Painel INVENTARIO mostra itens (Pocao Pequena x3, Espada Curta x1, etc.)
  - Painel CRAFT mostra 3 receitas

- [ ] **Verificar navegação:**
  - `1` → foco no PERSONAGEM (borda amarela)
  - `2` → foco no INVENTARIO, `↑↓` move o cursor `▶`
  - `3` → foco no CRAFT, `↑↓` move o cursor `▶`
  - `Tab` cicla entre painéis

- [ ] **Verificar craft:**
  - No painel CRAFT, selecionar "Pocao Media" (requer Pocao Pequena x2)
  - Pressionar `Enter`
  - Chat mostra `[sistema] Pocao Media craftado com sucesso!`
  - Painel INVENTARIO atualiza: Pocao Pequena x1, Pocao Media x1

- [ ] **Verificar craft sem ingredientes:**
  - Selecionar "Espada Longa" (requer Madeira x5 + Pedra x3)
  - Com Madeira x12 e Pedra x6 disponíveis → sucesso
  - Craftar de novo → falha com "Ingredientes insuficientes."

- [ ] **Verificar chat:**
  - Digitar mensagem, pressionar `Enter` → aparece no CHAT
  - Abrir segundo cliente → mensagens aparecem em ambos

- [ ] **Commit final:**
```bash
git add -A
git commit -m "feat: Killian MMORPG POC — character, inventory, craft via WebSocket"
```
