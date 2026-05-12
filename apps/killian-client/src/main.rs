mod model;
mod net;
mod view;
mod view_model;

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use killian_protocol::ClientMsg;
use model::{AppModel, GamePanel, InputMode, ReconnectState, Screen};
use tokio::sync::mpsc::error::TryRecvError;
use view_model::AppViewModel;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;

enum AppAction {
    None,
    TryConnect,
    SendChat,
    DoCraft,
    StartGather,
    CancelGather,
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

        // Tick gather: send to server when complete
        if let Some(action_id) = model.take_completed_gather() {
            model.push_chat_system("Coleta concluida! Aguardando servidor...".to_string());
            if let Some(net) = net_handle.as_ref() {
                let _ = net.tx.send(ClientMsg::Gather { action_id });
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
    match key.code {
        KeyCode::Esc => { model.should_quit = true; AppAction::None }
        KeyCode::Char('i') => { model.enter_insert_mode(); AppAction::None }
        KeyCode::Char('1') => { model.set_panel(GamePanel::Character); AppAction::None }
        KeyCode::Char('2') => { model.set_panel(GamePanel::Inventory); AppAction::None }
        KeyCode::Char('3') => { model.set_panel(GamePanel::Gather);  AppAction::None }
        KeyCode::Char('4') => { model.set_panel(GamePanel::Craft);   AppAction::None }
        KeyCode::Char('5') => { model.set_panel(GamePanel::Players); AppAction::None }
        KeyCode::Tab => { model.cycle_panel_focus(); AppAction::None }
        KeyCode::Up => { model.cursor_up(); AppAction::None }
        KeyCode::Down => { model.cursor_down(); AppAction::None }
        KeyCode::Enter => {
            match model.game.panel_focus {
                GamePanel::Craft   => AppAction::DoCraft,
                GamePanel::Gather  => AppAction::StartGather,
                _                  => AppAction::None,
            }
        }
        KeyCode::Char('x') => AppAction::CancelGather,
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
