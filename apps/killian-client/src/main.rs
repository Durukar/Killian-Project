mod credentials;
mod model;
mod net;
mod view;
mod view_model;

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use killian_protocol::ClientMsg;
use model::{quest_id_for_npc, AppModel, CreationFocus, GamePanel, InputMode, ReconnectState, Screen, PROFESSIONS, RACES};
use tokio::sync::mpsc::error::TryRecvError;
use view_model::AppViewModel;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;

enum AppAction {
    None,
    TryConnect,
    SendCreateCharacter,
    SendChat,
    DoCraft,
    StartGather,
    CancelGather,
    StartCombat,
    CancelCombat,
    Travel,
    ToggleMap,
    ToggleChar,
    UseItem,
    AllocStat,
    TalkToNpc,
    ToggleEquip,
    QuestInteract,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let saved = credentials::load();

    let default_nick = std::env::var("KILLIAN_NICK")
        .ok()
        .or_else(|| saved.as_ref().map(|s| s.nick.clone()))
        .unwrap_or_default();

    let default_password = saved.as_ref()
        .map(|s| s.password.clone())
        .unwrap_or_default();

    let default_server = std::env::var("KILLIAN_SERVER")
        .ok()
        .or_else(|| saved.as_ref().map(|s| s.server.clone()))
        .unwrap_or_else(|| "wss://killian.spellbook.app.br".to_string());

    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, AppModel::new(default_nick, default_password, default_server)).await;
    ratatui::restore();
    result
}

async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    mut model: AppModel,
) -> anyhow::Result<()> {
    let mut net_handle: Option<net::NetHandle> = None;

    loop {
        // Drain incoming server messages
        if let Some(net) = net_handle.as_mut() {
            loop {
                match net.rx.try_recv() {
                    Ok(msg) => model.push_server_msg(msg),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        net_handle = None;
                        if model.connecting {
                            // Server rejected or closed before sending CharacterUpdate
                            model.connecting = false;
                            model.screen = Screen::Connect;
                            model.connect.notices.push(
                                "Erro: servidor nao respondeu.".to_string()
                            );
                        } else if model.screen == Screen::Game {
                            model.start_reconnect();
                        } else {
                            model.on_disconnect();
                        }
                        break;
                    }
                }
            }
        }

        // Auto-reconnect
        if net_handle.is_none() {
            if let Some(ref rs) = model.reconnect {
                if Instant::now() >= rs.next_at {
                    let nick = rs.nick.clone();
                    let password = rs.password.clone();
                    let server = rs.server.clone();
                    let attempt = rs.attempts;
                    model.push_chat_system(format!("Reconectando... (tentativa {}/{})", attempt + 1, MAX_RECONNECT_ATTEMPTS));
                    match net::connect(&server, nick.clone(), password.clone()).await {
                        Ok(handle) => {
                            net_handle = Some(handle);
                            model.connecting = true;
                            model.reconnect = None;
                        }
                        Err(_) => {
                            if attempt + 1 >= MAX_RECONNECT_ATTEMPTS {
                                model.on_reconnect_failed();
                            } else {
                                let delay = Duration::from_secs(2u64.pow(attempt + 1));
                                model.push_chat_system(format!(
                                    "Falha. Proxima tentativa em {}s...",
                                    delay.as_secs()
                                ));
                                model.reconnect = Some(ReconnectState {
                                    nick,
                                    password,
                                    server,
                                    attempts: attempt + 1,
                                    next_at: Instant::now() + delay,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Save credentials on first successful login
        if model.just_logged_in {
            model.just_logged_in = false;
            credentials::save(
                &model.connect.nick,
                &model.connect.password,
                &model.connect.server,
            );
        }

        // Tick gather: send to server when complete
        if let Some(action_id) = model.take_completed_gather() {
            model.push_chat_system("Coleta concluida! Aguardando servidor...".to_string());
            if let Some(net) = net_handle.as_ref() {
                let _ = net.tx.send(ClientMsg::Gather { action_id });
            }
        }

        // Tick combat: send to server when complete
        if let Some(mob_id) = model.take_completed_combat() {
            model.push_chat_system("Combate concluido! Aguardando servidor...".to_string());
            if let Some(net) = net_handle.as_ref() {
                let _ = net.tx.send(ClientMsg::Attack { mob_id });
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
        Screen::CharacterCreation => handle_creation_key(key, model),
        Screen::Game => handle_game_key(key, model),
    }
}

fn handle_creation_key(key: crossterm::event::KeyEvent, model: &mut AppModel) -> AppAction {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            model.should_quit = true;
            AppAction::None
        }
        KeyCode::Tab | KeyCode::BackTab => {
            model.creation.focus = match model.creation.focus {
                CreationFocus::Race       => CreationFocus::Profession,
                CreationFocus::Profession => CreationFocus::Race,
            };
            AppAction::None
        }
        KeyCode::Up => {
            match model.creation.focus {
                CreationFocus::Race => {
                    model.creation.race_cursor = model.creation.race_cursor.saturating_sub(1);
                }
                CreationFocus::Profession => {
                    model.creation.profession_cursor = model.creation.profession_cursor.saturating_sub(1);
                }
            }
            AppAction::None
        }
        KeyCode::Down => {
            match model.creation.focus {
                CreationFocus::Race => {
                    model.creation.race_cursor = (model.creation.race_cursor + 1).min(RACES.len() - 1);
                }
                CreationFocus::Profession => {
                    model.creation.profession_cursor = (model.creation.profession_cursor + 1).min(PROFESSIONS.len() - 1);
                }
            }
            AppAction::None
        }
        KeyCode::Enter => AppAction::SendCreateCharacter,
        _ => AppAction::None,
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
    // Ctrl+C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        model.should_quit = true;
        return AppAction::None;
    }

    match model.game.input_mode {
        InputMode::Insert => handle_insert_key(key, model),
        InputMode::Normal => handle_normal_key(key, model),
    }
}

fn handle_insert_key(key: crossterm::event::KeyEvent, model: &mut AppModel) -> AppAction {
    match key.code {
        KeyCode::Esc => { model.enter_normal_mode(); AppAction::None }
        KeyCode::Enter => AppAction::SendChat,
        KeyCode::Backspace => { model.pop_chat_char(); AppAction::None }
        KeyCode::Char(ch) => { model.push_chat_char(ch); AppAction::None }
        _ => AppAction::None,
    }
}

fn handle_normal_key(key: crossterm::event::KeyEvent, model: &mut AppModel) -> AppAction {
    // Character popup intercepts most keys while open
    if model.game.char_open {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('1') => { model.close_char(); AppAction::None }
            KeyCode::Up   => { model.game.stat_cursor = model.game.stat_cursor.saturating_sub(1); AppAction::None }
            KeyCode::Down => { model.game.stat_cursor = (model.game.stat_cursor + 1).min(3); AppAction::None }
            KeyCode::Char('a') => AppAction::AllocStat,
            _ => AppAction::None,
        };
    }

    // Map popup intercepts most keys while open
    if model.game.map_open {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('m') => { model.close_map(); AppAction::None }
            KeyCode::Up    => { model.map_move(0, -1); AppAction::None }
            KeyCode::Down  => { model.map_move(0,  1); AppAction::None }
            KeyCode::Left  => { model.map_move(-1, 0); AppAction::None }
            KeyCode::Right => { model.map_move(1,  0); AppAction::None }
            KeyCode::Enter => AppAction::Travel,
            _ => AppAction::None,
        };
    }

    match key.code {
        KeyCode::Esc => { model.should_quit = true; AppAction::None }
        KeyCode::Char('i') => { model.enter_insert_mode(); AppAction::None }
        KeyCode::Char('m') => AppAction::ToggleMap,
        KeyCode::Char('1') => AppAction::ToggleChar,
        KeyCode::Char('2') => { model.set_panel(GamePanel::Inventory); AppAction::None }
        KeyCode::Char('3') => { model.set_panel(GamePanel::Map);      AppAction::None }
        KeyCode::Char('4') => {
            if model.game.panel_focus == GamePanel::Gather {
                model.set_panel(GamePanel::Character);
            } else {
                model.set_panel(GamePanel::Gather);
            }
            AppAction::None
        }
        KeyCode::Char('5') => {
            if model.game.panel_focus == GamePanel::Combat {
                model.set_panel(GamePanel::Character);
            } else {
                model.set_panel(GamePanel::Combat);
            }
            AppAction::None
        }
        KeyCode::Char('6') => {
            if model.game.panel_focus == GamePanel::Craft {
                model.set_panel(GamePanel::Character);
            } else {
                model.set_panel(GamePanel::Craft);
            }
            AppAction::None
        }
        KeyCode::Char('7') => { model.set_panel(GamePanel::Players); AppAction::None }
        KeyCode::Char('8') => { model.set_panel(GamePanel::Npcs);   AppAction::None }
        KeyCode::Tab => { model.cycle_panel_focus(); AppAction::None }
        KeyCode::Up => { model.cursor_up(); AppAction::None }
        KeyCode::Down => { model.cursor_down(); AppAction::None }
        KeyCode::Enter => {
            match model.game.panel_focus {
                GamePanel::Craft   => AppAction::DoCraft,
                GamePanel::Gather  => AppAction::StartGather,
                GamePanel::Combat  => AppAction::StartCombat,
                GamePanel::Map     => AppAction::ToggleMap,
                GamePanel::Npcs    => AppAction::TalkToNpc,
                _                  => AppAction::None,
            }
        }
        KeyCode::Char('q') => {
            if model.game.panel_focus == GamePanel::Npcs {
                AppAction::QuestInteract
            } else {
                AppAction::None
            }
        }
        KeyCode::Char('x') => {
            if model.game.combat.is_some() {
                AppAction::CancelCombat
            } else {
                AppAction::CancelGather
            }
        }
        KeyCode::Char('u') => {
            if model.game.panel_focus == crate::model::GamePanel::Inventory {
                AppAction::UseItem
            } else {
                AppAction::None
            }
        }
        KeyCode::Char('e') => {
            if model.game.panel_focus == crate::model::GamePanel::Inventory {
                AppAction::ToggleEquip
            } else {
                AppAction::None
            }
        }
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
        AppAction::SendCreateCharacter => {
            let race = model.selected_race();
            let profession = model.selected_profession();
            let Some(net) = net_handle.as_ref() else { return };
            let _ = net.tx.send(ClientMsg::CreateCharacter { race, profession });
        }
        AppAction::TryConnect => {
            if !model.can_connect() {
                model.on_connect_error("Nick e servidor sao obrigatorios.".to_string());
                return;
            }
            let (nick, password, server) = model.connect_payload();
            model.connect.notices.push(format!("Conectando em {server} ..."));
            match net::connect(&server, nick, password).await {
                Ok(handle) => {
                    *net_handle = Some(handle);
                    model.on_ws_connected();
                }
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
        AppAction::StartGather => {
            model.start_gather();
        }
        AppAction::CancelGather => {
            model.cancel_gather();
        }
        AppAction::StartCombat => {
            model.start_combat();
        }
        AppAction::CancelCombat => {
            model.cancel_combat();
        }
        AppAction::Travel => {
            if let Some(zone_id) = model.travel_to_selected_zone() {
                if let Some(net) = net_handle.as_ref() {
                    let _ = net.tx.send(ClientMsg::Travel { zone_id });
                }
            }
        }
        AppAction::ToggleMap => {
            model.toggle_map();
        }
        AppAction::ToggleChar => {
            model.toggle_char();
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
        AppAction::UseItem => {
            let Some(item_name) = model.selected_item_name() else { return };
            let Some(net) = net_handle.as_ref() else {
                model.push_chat_system("sem conexao ativa".to_string());
                return;
            };
            if net.tx.send(ClientMsg::UseItem { item_name }).is_err() {
                model.push_chat_system("falha ao usar item".to_string());
            }
        }
        AppAction::TalkToNpc => {
            model.talk_to_npc();
        }
        AppAction::ToggleEquip => {
            let Some(item_name) = model.selected_item_name() else { return };
            let Some(net) = net_handle.as_ref() else {
                model.push_chat_system("sem conexao ativa".to_string());
                return;
            };
            let msg = if model.selected_item_is_equipped() {
                ClientMsg::Unequip { item_name }
            } else {
                ClientMsg::Equip { item_name }
            };
            if net.tx.send(msg).is_err() {
                model.push_chat_system("falha ao enviar equipamento".to_string());
            }
        }
        AppAction::AllocStat => {
            let stat = model.selected_stat();
            let Some(net) = net_handle.as_ref() else {
                model.push_chat_system("sem conexao ativa".to_string());
                return;
            };
            if net.tx.send(ClientMsg::AllocStat { stat }).is_err() {
                model.push_chat_system("falha ao alocar atributo".to_string());
            }
        }
        AppAction::QuestInteract => {
            let npcs = model.npcs_for_zone();
            let Some(npc) = npcs.get(model.game.npc_cursor) else { return };
            let npc_name = npc.name.to_string();
            let Some(net) = net_handle.as_ref() else {
                model.push_chat_system("sem conexao ativa".to_string());
                return;
            };
            // Check if NPC has a completable turn-in quest first
            if let Some(q) = model.game.quests.iter().find(|q| q.giver == npc_name && q.can_turn_in) {
                let quest_id = q.id.clone();
                let _ = net.tx.send(ClientMsg::TurnInQuest { quest_id });
            } else if model.game.quests.iter().any(|q| q.giver == npc_name) {
                model.push_chat_system(format!("{}: Complete a missão antes de entregar.", npc_name));
            } else if let Some(quest_id) = quest_id_for_npc(&npc_name) {
                let _ = net.tx.send(ClientMsg::AcceptQuest { quest_id: quest_id.to_string() });
            } else {
                model.push_chat_system(format!("{}: Não tenho missões para você agora.", npc_name));
            }
        }
    }
}
