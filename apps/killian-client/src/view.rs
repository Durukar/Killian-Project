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

    let header_text = format!(
        " {} @ {}    [1] Personagem  [2] Inventario  [3] Craft",
        vm.nick, vm.server
    );
    frame.render_widget(
        Paragraph::new(header_text)
            .block(Block::default().borders(Borders::BOTTOM)),
        root[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(0)])
        .split(root[1]);

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
