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
