# Gather System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adicionar um painel `[5] COLETA` com ações de coleta de recursos que mostram barra de progresso com delay, enviando o pedido ao servidor apenas quando o tempo termina.

**Architecture:** O delay é puramente visual no cliente (Instant + Duration). Quando o timer chega a zero, o cliente envia `ClientMsg::Gather { action_id }`. O servidor valida a ação, aplica um yield aleatório ao inventário, salva e responde com `InventoryUpdate` + mensagem no chat. O servidor é a fonte de verdade do inventário — o cliente só controla o tempo de espera.

**Tech Stack:** Rust, ratatui 0.30, tokio, rand 0.9 (novo no servidor), crossterm

---

## File Map

| Arquivo | Mudança |
|---------|---------|
| `apps/killian-server/Cargo.toml` | Adicionar `rand = "0.9"` |
| `crates/killian-protocol/src/lib.rs` | `ClientMsg::Gather`, `ServerMsg::GatherResult` |
| `apps/killian-server/src/gather.rs` | **CRIAR** — ações de coleta, `apply_gather` com rand |
| `apps/killian-server/src/main.rs` | Importar gather.rs, tratar `ClientMsg::Gather` |
| `apps/killian-client/src/model.rs` | `GatherAction`, `GatherProgress`, `GamePanel::Gather`, estado e métodos |
| `apps/killian-client/src/view_model.rs` | Campos de gather no `GameViewModel` |
| `apps/killian-client/src/view.rs` | Painel `[5] COLETA` com lista e barra de progresso |
| `apps/killian-client/src/main.rs` | Tecla `5`, tick de gather no loop principal |

---

### Task 1: Protocolo — Gather e GatherResult

**Files:**
- Modify: `crates/killian-protocol/src/lib.rs`

- [ ] **Adicionar `Gather` ao `ClientMsg` e `GatherResult` ao `ServerMsg`:**

```rust
// Em ClientMsg, após Craft:
Craft { recipe_id: String },
Gather { action_id: String },

// Em ServerMsg, após CraftResult:
CraftResult { success: bool, message: String },
GatherResult { message: String, items: Vec<InventoryItem> },
```

- [ ] **Verificar:**
```bash
cargo check -p killian-protocol
```
Expected: sem erros.

- [ ] **Commit:**
```bash
git add crates/killian-protocol/src/lib.rs
git commit -m "feat(protocol): add Gather and GatherResult messages"
```

---

### Task 2: Servidor — rand + gather.rs

**Files:**
- Modify: `apps/killian-server/Cargo.toml`
- Create: `apps/killian-server/src/gather.rs`

- [ ] **Adicionar rand ao `apps/killian-server/Cargo.toml`:**

```toml
[package]
name = "killian-server"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
killian-protocol = { path = "../../crates/killian-protocol" }
futures-util = "0.3"
rand = "0.9"
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.28"
```

- [ ] **Criar `apps/killian-server/src/gather.rs`:**

```rust
use killian_protocol::InventoryItem;
use rand::Rng;

pub struct GatherAction {
    pub id: &'static str,
    pub name: &'static str,
    pub location: &'static str,
    pub yield_item: &'static str,
    pub min_qty: u32,
    pub max_qty: u32,
}

pub fn all_gather_actions() -> &'static [GatherAction] {
    &[
        GatherAction { id: "coletar_madeira",  name: "Coletar Madeira",  location: "Floresta", yield_item: "Madeira",       min_qty: 2, max_qty: 4 },
        GatherAction { id: "coletar_galhos",   name: "Coletar Galhos",   location: "Floresta", yield_item: "Madeira",       min_qty: 1, max_qty: 2 },
        GatherAction { id: "minerar_pedra",    name: "Minerar Pedra",    location: "Mina",     yield_item: "Pedra",         min_qty: 2, max_qty: 3 },
        GatherAction { id: "minerar_mineral",  name: "Minerar Mineral",  location: "Mina",     yield_item: "Pedra",         min_qty: 4, max_qty: 6 },
        GatherAction { id: "colher_ervas",     name: "Colher Ervas",     location: "Campos",   yield_item: "Pocao Pequena", min_qty: 1, max_qty: 1 },
    ]
}

pub fn apply_gather(inventory: &mut Vec<InventoryItem>, action: &GatherAction) -> Vec<InventoryItem> {
    let qty = rand::rng().random_range(action.min_qty..=action.max_qty);
    if let Some(existing) = inventory.iter_mut().find(|i| i.name == action.yield_item) {
        existing.qty += qty;
    } else {
        inventory.push(InventoryItem { name: action.yield_item.to_string(), qty });
    }
    vec![InventoryItem { name: action.yield_item.to_string(), qty }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_gather_adds_item_to_empty_inventory() {
        let mut inv = vec![];
        let action = &all_gather_actions()[0]; // coletar_madeira
        let yielded = apply_gather(&mut inv, action);
        assert!(!inv.is_empty());
        assert_eq!(inv[0].name, "Madeira");
        assert!(inv[0].qty >= action.min_qty && inv[0].qty <= action.max_qty);
        assert_eq!(yielded[0].name, "Madeira");
    }

    #[test]
    fn apply_gather_stacks_on_existing_item() {
        let mut inv = vec![InventoryItem { name: "Madeira".to_string(), qty: 5 }];
        let action = &all_gather_actions()[0]; // coletar_madeira
        apply_gather(&mut inv, action);
        assert_eq!(inv.len(), 1);
        assert!(inv[0].qty >= 5 + action.min_qty);
    }

    #[test]
    fn apply_gather_creates_new_item_when_not_present() {
        let mut inv = vec![InventoryItem { name: "Pedra".to_string(), qty: 3 }];
        let action = &all_gather_actions()[0]; // coletar_madeira (Madeira)
        apply_gather(&mut inv, action);
        assert_eq!(inv.len(), 2);
        assert!(inv.iter().any(|i| i.name == "Madeira"));
    }
}
```

- [ ] **Rodar testes:**
```bash
cargo test -p killian-server
```
Expected: 7 testes passando (4 de craft + 3 novos de gather).

- [ ] **Commit:**
```bash
git add apps/killian-server/Cargo.toml apps/killian-server/src/gather.rs
git commit -m "feat(server): add gather actions with random yield"
```

---

### Task 3: Servidor — handle ClientMsg::Gather

**Files:**
- Modify: `apps/killian-server/src/main.rs`

- [ ] **Adicionar `mod gather;` no topo e importar `apply_gather`, `all_gather_actions`:**

Adicionar após `mod craft;`:
```rust
mod gather;
```

Adicionar ao `use craft::...`:
```rust
use gather::{all_gather_actions, apply_gather};
```

- [ ] **Adicionar tratamento de `ClientMsg::Gather` dentro do `match serde_json::from_str::<ClientMsg>` no loop principal**, após o bloco de `ClientMsg::Craft`:

```rust
Ok(ClientMsg::Gather { action_id }) => {
    let gather_actions = all_gather_actions();
    let result = if let Some(action) = gather_actions.iter().find(|a| a.id == action_id) {
        let yielded = apply_gather(&mut inventory, action);
        persistence::save_inventory(&nick, &inventory);
        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
            items: inventory.clone(),
        }).await?;
        let items_desc = yielded.iter()
            .map(|i| format!("{} x{}", i.name, i.qty))
            .collect::<Vec<_>>()
            .join(", ");
        ServerMsg::GatherResult {
            message: format!("Voce coletou: {items_desc}"),
            items: yielded,
        }
    } else {
        ServerMsg::GatherResult {
            message: "Acao de coleta desconhecida.".to_string(),
            items: vec![],
        }
    };
    send_msg(&mut ws_writer, &result).await?;
}
```

- [ ] **Verificar:**
```bash
cargo check -p killian-server
```
Expected: sem erros.

- [ ] **Commit:**
```bash
git add apps/killian-server/src/main.rs
git commit -m "feat(server): handle Gather message, apply yield and save inventory"
```

---

### Task 4: Client model — GatherAction, GatherProgress, GamePanel::Gather

**Files:**
- Modify: `apps/killian-client/src/model.rs`

- [ ] **Adicionar structs e enum no topo do arquivo**, após os imports:

```rust
use std::time::{Duration, Instant};
// (já existe, não duplicar)

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
```

- [ ] **Adicionar `GamePanel::Gather` ao enum `GamePanel`** e atualizar `next()`:

```rust
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
```

- [ ] **Adicionar campos ao `GameState`:**

```rust
pub gather_cursor: usize,
pub gathering: Option<GatherProgress>,
```

- [ ] **Inicializar no `AppModel::new`:**

```rust
gather_cursor: 0,
gathering: None,
```

- [ ] **Adicionar métodos ao `AppModel`:**

```rust
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
```

- [ ] **Atualizar `cursor_up` e `cursor_down` para `GamePanel::Gather`:**

```rust
// Em cursor_up:
GamePanel::Gather => {
    self.game.gather_cursor = self.game.gather_cursor.saturating_sub(1);
}

// Em cursor_down:
GamePanel::Gather => {
    let max = all_gather_actions().len().saturating_sub(1);
    self.game.gather_cursor = (self.game.gather_cursor + 1).min(max);
}
```

- [ ] **Tratar `ServerMsg::GatherResult` em `push_server_msg`**, após `CraftResult`:

```rust
ServerMsg::GatherResult { message, items: _ } => {
    self.push_chat_system(message);
}
```

- [ ] **Verificar:**
```bash
cargo check -p killian-client 2>&1 | head -20
```
Expected: erros nos outros módulos do cliente — normal neste passo.

- [ ] **Commit:**
```bash
git add apps/killian-client/src/model.rs
git commit -m "feat(client/model): GatherAction, GatherProgress, GamePanel::Gather"
```

---

### Task 5: Client viewmodel — campos de gather

**Files:**
- Modify: `apps/killian-client/src/view_model.rs`

- [ ] **Adicionar ao `GameViewModel`:**

```rust
pub gather_cursor: usize,
pub gathering: Option<GatherViewProgress>,
```

- [ ] **Adicionar struct `GatherViewProgress`** (versão serializável sem `Instant` para a view):

```rust
pub struct GatherViewProgress {
    pub action_name: String,
    pub location: String,
    pub ratio: f64,
    pub elapsed_secs: u64,
    pub total_secs: u64,
}
```

- [ ] **Atualizar o `From<&AppModel>` para `AppViewModel::Game`:**

```rust
gather_cursor: model.game.gather_cursor,
gathering: model.game.gathering.as_ref().map(|g| GatherViewProgress {
    action_name: g.action_name.clone(),
    location: g.location.clone(),
    ratio: g.ratio(),
    elapsed_secs: g.elapsed_secs(),
    total_secs: g.total_secs(),
}),
```

- [ ] **Commit:**
```bash
git add apps/killian-client/src/view_model.rs
git commit -m "feat(client/viewmodel): add gather fields"
```

---

### Task 6: Client view — painel [5] COLETA

**Files:**
- Modify: `apps/killian-client/src/view.rs`

- [ ] **Adicionar import de `GatherAction` e `all_gather_actions`:**

```rust
use crate::model::{all_gather_actions, ConnectField, GamePanel, InputMode};
```

- [ ] **Atualizar o header da game screen** para incluir `[5] Coleta`:

```rust
let header_text = format!(
    " {} @ {}    [1] Personagem  [2] Inventario  [3] Coleta  [4] Craft  [5] Online",
    vm.nick, vm.server
);
```

- [ ] **Atualizar o layout da coluna esquerda** para 5 painéis:

```rust
let left = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(9),    // PERSONAGEM
        Constraint::Length(7),    // INVENTARIO
        Constraint::Length(9),    // COLETA
        Constraint::Min(0),       // CRAFT
        Constraint::Length(6),    // ONLINE
    ])
    .split(body[0]);
```

- [ ] **Atualizar as chamadas de render** para incluir o novo painel na posição correta:

```rust
render_character_panel(frame, left[0], vm);
render_inventory_panel(frame, left[1], vm);
render_gather_panel(frame, left[2], vm);
render_craft_panel(frame, left[3], vm);
render_players_panel(frame, left[4], vm);
render_chat_panel(frame, body[1], vm);
```

- [ ] **Atualizar hints no modo Normal** para incluir a nova tecla:

```rust
"i: digitar | 1-5: painel | Tab: cicla | ↑↓: navegar | Enter: acao | x: cancelar coleta | Esc: sai",
```

- [ ] **Adicionar a função `render_gather_panel`:**

```rust
fn render_gather_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Gather;
    let border_style = focus_style(focused);

    if let Some(ref progress) = vm.gathering {
        // Progress mode
        let bar_width = area.width.saturating_sub(4) as usize;
        let filled = (progress.ratio * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let lines = vec![
            Line::from(Span::styled(
                format!(" ⟳ {} ({})", progress.action_name, progress.location),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::raw(format!(" {}", bar)),
            Line::raw(format!("  {}/{}s", progress.elapsed_secs, progress.total_secs)),
            Line::from(Span::styled(
                " x: cancelar",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title("[3] COLETA"),
            ),
            area,
        );
        return;
    }

    // List mode
    let actions = all_gather_actions();
    let h = area.height.saturating_sub(2) as usize;
    let cursor = vm.gather_cursor;
    let start = if cursor >= h { cursor - h + 1 } else { 0 };
    let end = (start + h).min(actions.len());

    let lines: Vec<Line<'_>> = actions[start..end]
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let idx = start + i;
            let prefix = if idx == cursor { "▶ " } else { "  " };
            let style = if idx == cursor && focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(
                format!("{}{}  {}s", prefix, action.name, action.duration_secs),
                style,
            ))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("[3] COLETA (Enter: iniciar)"),
        ),
        area,
    );
}
```

- [ ] **Verificar:**
```bash
cargo check -p killian-client 2>&1 | head -20
```
Expected: erros em `main.rs` por `GamePanel::Gather` não tratado — normal.

- [ ] **Commit:**
```bash
git add apps/killian-client/src/view.rs
git commit -m "feat(client/view): gather panel with progress bar"
```

---

### Task 7: Client main — tecla 5, tick gather, send Gather

**Files:**
- Modify: `apps/killian-client/src/main.rs`

- [ ] **Adicionar `GamePanel::Gather` no `handle_normal_key`:**

```rust
KeyCode::Char('3') => { model.set_panel(GamePanel::Gather);    AppAction::None }
KeyCode::Char('4') => { model.set_panel(GamePanel::Craft);     AppAction::None }
KeyCode::Char('5') => { model.set_panel(GamePanel::Players);   AppAction::None }
```

- [ ] **Adicionar `AppAction::StartGather` e `AppAction::CancelGather`** ao enum:

```rust
enum AppAction {
    None,
    TryConnect,
    SendChat,
    DoCraft,
    StartGather,
    CancelGather,
}
```

- [ ] **Atualizar `handle_normal_key` — `Enter` agora depende do painel, e `x` cancela:**

```rust
KeyCode::Enter => {
    match model.game.panel_focus {
        GamePanel::Craft   => AppAction::DoCraft,
        GamePanel::Gather  => AppAction::StartGather,
        _                  => AppAction::None,
    }
}
KeyCode::Char('x') => AppAction::CancelGather,
```

- [ ] **Adicionar tick de gather no loop principal**, logo após o bloco de auto-reconnect e antes do `terminal.draw`:

```rust
// Tick gather: send to server when complete
if let Some(action_id) = model.take_completed_gather() {
    model.push_chat_system("Coleta concluida! Aguardando servidor...".to_string());
    if let Some(net) = net_handle.as_ref() {
        let _ = net.tx.send(ClientMsg::Gather { action_id });
    }
}
```

- [ ] **Adicionar tratamento de `StartGather` e `CancelGather` em `process_action`:**

```rust
AppAction::StartGather => {
    model.start_gather();
}
AppAction::CancelGather => {
    model.cancel_gather();
}
```

- [ ] **Build final:**
```bash
cargo build --workspace
```
Expected: `Finished` sem erros.

- [ ] **Rodar todos os testes:**
```bash
cargo test --workspace
```
Expected: 7 testes passando (4 craft + 3 gather).

- [ ] **Commit:**
```bash
git add apps/killian-client/src/main.rs
git commit -m "feat(client): gather panel keybindings, progress tick, send on complete"
```

---

### Task 8: Teste manual end-to-end

- [ ] **Terminal 1:**
```bash
cargo run -p killian-server
```

- [ ] **Terminal 2:**
```bash
cargo run -p killian-client
```

- [ ] **Verificar painel COLETA:**
  - `3` foca o painel (borda amarela)
  - Lista mostra: Coletar Madeira 8s, Coletar Galhos 4s, etc.
  - `↑↓` navega a lista

- [ ] **Verificar gather com progresso:**
  - Selecionar "Coletar Galhos" (4s — mais rápido pra testar)
  - `Enter` inicia
  - Painel muda para modo progresso: barra `████░░░░`, `2/4s`
  - Chat mostra "Iniciando: Coletar Galhos (Floresta)"
  - Após 4s: chat mostra "Coleta concluida!" e depois "Voce coletou: Madeira x1" ou x2
  - Inventário atualiza com o novo item

- [ ] **Verificar cancelamento:**
  - Iniciar "Coletar Madeira" (8s)
  - Pressionar `x`
  - Chat mostra "Coleta cancelada."
  - Painel volta para lista

- [ ] **Verificar que não pode coletar durante coleta:**
  - Iniciar coleta
  - Pressionar `Enter` novamente
  - Nada acontece (o método `start_gather` retorna early se `gathering.is_some()`)

- [ ] **Verificar persistência:**
  - Coletar Madeira algumas vezes
  - Sair e reconectar
  - Inventário deve ter a madeira coletada
