use killian_protocol::{ItemType, Rarity};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::model::{ConnectField, GamePanel, InputMode};
use crate::view_model::{AppViewModel, CombatViewProgress, ConnectViewModel, GameViewModel, GatherViewProgress};

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

    let password_masked = "*".repeat(vm.password_len);
    let password_style = focus_style(vm.focus == ConnectField::Password);
    frame.render_widget(
        Paragraph::new(password_masked.as_str()).style(password_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(password_style)
                .title("Senha"),
        ),
        areas[2],
    );

    let server_style = focus_style(vm.focus == ConnectField::Server);
    frame.render_widget(
        Paragraph::new(vm.server.as_str()).style(server_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(server_style)
                .title("Servidor WS (ex: ws://192.168.1.22:7001)"),
        ),
        areas[3],
    );

    let lines: Vec<Line<'_>> = vm.notices.iter().map(|l| Line::raw(l.as_str())).collect();
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Status (Enter conecta | Esc sai)"))
            .wrap(Wrap { trim: false }),
        areas[4],
    );

    let cursor = match vm.focus {
        ConnectField::Nick      => (areas[1], vm.nick.chars().count()),
        ConnectField::Password  => (areas[2], vm.password_len),
        ConnectField::Server    => (areas[3], vm.server.chars().count()),
    };
    set_cursor(frame, cursor.0, cursor.1);
}

fn render_game(frame: &mut Frame, vm: &GameViewModel) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header: KILLIAN // nick · zona  |  HP · MP · Gold
    let zone_name = vm.zones.iter().find(|z| z.is_current).map(|z| z.name).unwrap_or("?");
    let header_line = if let Some(ch) = &vm.character {
        let hp_ratio = ch.hp as f64 / ch.max_hp.max(1) as f64;
        let hp_color = if hp_ratio > 0.5 { Color::Green } else if hp_ratio > 0.25 { Color::Yellow } else { Color::Red };
        Line::from(vec![
            Span::styled(" KILLIAN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("// ", Style::default().fg(Color::DarkGray)),
            Span::styled(vm.nick.as_str(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("  ·  ", Style::default().fg(Color::DarkGray)),
            Span::styled(zone_name, Style::default().fg(Color::Yellow)),
            Span::styled("     ♥ ", Style::default().fg(hp_color)),
            Span::styled(format!("{}/{}", ch.hp, ch.max_hp), Style::default().fg(hp_color).add_modifier(Modifier::BOLD)),
            Span::styled("   ◆ ", Style::default().fg(Color::Blue)),
            Span::styled(format!("{}/{}", ch.mp, ch.max_mp), Style::default().fg(Color::Blue)),
            Span::styled("   ● ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{}g", ch.gold), Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" KILLIAN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("// ", Style::default().fg(Color::DarkGray)),
            Span::styled(vm.nick.as_str(), Style::default().fg(Color::White)),
            Span::styled("  ·  ", Style::default().fg(Color::DarkGray)),
            Span::styled(zone_name, Style::default().fg(Color::Yellow)),
        ])
    };
    frame.render_widget(
        Paragraph::new(vec![Line::raw(""), header_line])
            .block(Block::default().borders(Borders::BOTTOM)),
        root[0],
    );

    render_xp_bar(frame, root[1], vm);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(0), Constraint::Length(24)])
        .split(root[2]);

    // Left: inventário (personagem virou popup via [1])
    render_inventory_panel(frame, body[0], vm);

    // Center: chat + log
    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(9)])
        .split(body[1]);
    render_chat_panel(frame, center[0], vm);
    render_game_log_panel(frame, center[1], vm);

    // Right: painel contextual
    render_right_column(frame, body[2], vm);

    // Barra de entrada
    let (mode_label, mode_style, hints) = match vm.input_mode {
        InputMode::Normal => (
            "[N]",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            "i:chat 1:char m:mapa 2-8:painéis e:equip u:usar ↑↓:nav Enter:ok Esc:sai",
        ),
        InputMode::Insert => (
            "[I]",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            "Enter: enviar | Esc: modo normal",
        ),
    };
    let input_title = format!("{} {}", mode_label, hints);
    frame.render_widget(
        Paragraph::new(vm.chat_input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(input_title, mode_style)),
            ),
        root[3],
    );
    if vm.input_mode == InputMode::Insert {
        set_cursor(frame, root[3], vm.chat_input.chars().count());
    }

    if vm.map_open {
        render_map_popup(frame, vm);
    }

    if vm.char_open {
        render_char_popup(frame, vm);
    }
}

fn render_xp_bar(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let line = if let Some(ch) = &vm.character {
        let bar_w = (area.width as usize).saturating_sub(22);
        let ratio = if ch.xp_next == 0 { 1.0 } else { (ch.xp as f64 / ch.xp_next as f64).clamp(0.0, 1.0) };
        let filled = (ratio * bar_w as f64) as usize;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_w.saturating_sub(filled)));
        Line::from(vec![
            Span::styled(format!(" Lv.{:<3}", ch.level), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(bar, Style::default().fg(Color::Cyan)),
            Span::styled(format!("  {}/{} XP", ch.xp, ch.xp_next), Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::raw("")
    };
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray))),
        area,
    );
}

fn render_right_column(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    match vm.panel_focus {
        GamePanel::Gather if vm.gathering.is_none() => render_gather_panel(frame, area, vm),
        GamePanel::Combat if vm.combat.is_none()    => render_combat_panel(frame, area, vm),
        GamePanel::Craft                            => render_craft_panel(frame, area, vm),
        GamePanel::Players                          => render_players_panel(frame, area, vm),
        GamePanel::Npcs                             => render_npc_panel(frame, area, vm),
        _                                           => render_actions_menu(frame, area, vm),
    }
}

fn render_actions_menu(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let zone_name = vm.zones.iter().find(|z| z.is_current).map(|z| z.name).unwrap_or("?");
    let key  = |s: &'static str| Span::styled(s, Style::default().fg(Color::DarkGray));
    let cat  = |s: &'static str, color: Color| {
        Line::from(Span::styled(s, Style::default().fg(color).add_modifier(Modifier::BOLD)))
    };
    let sep  = || Line::from(Span::styled(
        "─".repeat(area.width.saturating_sub(2) as usize),
        Style::default().fg(Color::DarkGray),
    ));

    let bar_w = area.width.saturating_sub(4) as usize;

    let gather_lines = if let Some(ref g) = vm.gathering {
        let filled = (g.ratio * bar_w as f64) as usize;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_w.saturating_sub(filled)));
        vec![
            Line::from(Span::styled(
                format!(" ⟳ {}  {}/{}s", g.action_name, g.elapsed_secs, g.total_secs),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(format!(" {}", bar), Style::default().fg(Color::Green))),
            Line::from(vec![key(" [x] "), Span::styled("cancelar", Style::default().fg(Color::Red))]),
        ]
    } else {
        vec![Line::from(vec![
            key(" [4] "),
            Span::styled("Coleta", Style::default().fg(Color::Green)),
        ])]
    };

    let combat_lines = if let Some(ref c) = vm.combat {
        let filled = (c.ratio * bar_w as f64) as usize;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_w.saturating_sub(filled)));
        vec![
            Line::from(Span::styled(
                format!(" ⚔ {}  {}/{}s", c.mob_name, c.elapsed_secs, c.total_secs),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(format!(" {}", bar), Style::default().fg(Color::Red))),
            Line::from(vec![key(" [x] "), Span::styled("cancelar", Style::default().fg(Color::Red))]),
        ]
    } else {
        vec![Line::from(vec![
            key(" [5] "),
            Span::styled("Combate", Style::default().fg(Color::Red)),
        ])]
    };

    let mut lines = vec![
        cat("◈ EXPLORAR", Color::Yellow),
        Line::from(vec![key(" [m] "), Span::styled(zone_name, Style::default().fg(Color::Yellow))]),
    ];
    lines.extend(gather_lines);
    lines.extend(combat_lines);
    lines.push(sep());
    lines.extend(vec![
        cat("◈ CRAFTAR", Color::Magenta),
        Line::from(vec![key(" [6] "), Span::styled("Receitas", Style::default().fg(Color::Magenta))]),
        sep(),
        cat("◈ SOCIAL", Color::Cyan),
        Line::from(vec![key(" [i] "), Span::styled("Chat", Style::default().fg(Color::Cyan))]),
        Line::from(vec![key(" [7] "), Span::styled("Online", Style::default().fg(Color::Green))]),
        Line::from(vec![key(" [8] "), Span::styled("NPCs", Style::default().fg(Color::Yellow))]),
        sep(),
        cat("◈ SISTEMA", Color::DarkGray),
        Line::from(vec![key(" [1] "), Span::raw("Personagem")]),
        Line::from(vec![key(" [Esc] "), Span::styled("Sair", Style::default().fg(Color::Red))]),
    ]);

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled("AÇÕES", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
        ),
        area,
    );
}

fn mini_bar(current: i32, max: i32, width: usize) -> String {
    let ratio = if max <= 0 { 0.0 } else { (current as f64 / max as f64).clamp(0.0, 1.0) };
    let filled = (ratio * width as f64) as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(width.saturating_sub(filled)))
}

fn hp_bar_color(hp: i32, max: i32) -> Color {
    if max <= 0 { return Color::DarkGray; }
    let pct = hp as f64 / max as f64;
    if pct > 0.5 { Color::Green } else if pct > 0.25 { Color::Yellow } else { Color::Red }
}

fn render_char_popup(frame: &mut Frame, vm: &GameViewModel) {
    let area = centered_rect(52, 16, frame.area());
    frame.render_widget(Clear, area);

    let bar_w: usize = 26;

    let lines: Vec<Line<'_>> = if let Some(ch) = &vm.character {
        let has_pts = ch.stat_points > 0;
        let sc = vm.stat_cursor;
        let hc = hp_bar_color(ch.hp, ch.max_hp);

        let xp_bar = mini_bar(ch.xp as i32, ch.xp_next.max(1) as i32, bar_w);
        let hp_bar  = mini_bar(ch.hp, ch.max_hp, bar_w);
        let mp_bar  = mini_bar(ch.mp, ch.max_mp, bar_w);

        let stat_col = |label: &'static str, val: u32, idx: usize, color: Color| -> Vec<Span<'static>> {
            let selected = has_pts && sc == idx;
            let val_style = if selected {
                Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            };
            vec![
                Span::styled(format!("   {}: ", label), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<3}", val), val_style),
            ]
        };

        let mut stat_spans: Vec<Span<'_>> = Vec::new();
        stat_spans.extend(stat_col("FOR", ch.str_stat, 0, Color::Red));
        stat_spans.extend(stat_col("DEF", ch.def_stat, 1, Color::Cyan));
        stat_spans.extend(stat_col("AGI", ch.agi_stat, 2, Color::Green));
        stat_spans.extend(stat_col("VIT", ch.vit_stat, 3, Color::Yellow));

        let pts_style = if has_pts {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        vec![
            Line::raw(""),
            Line::from(vec![
                Span::raw("   "),
                Span::styled(ch.class_name.as_str(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("   Lv.", Style::default().fg(Color::DarkGray)),
                Span::styled(ch.level.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("   XP ", Style::default().fg(Color::DarkGray)),
                Span::styled(xp_bar, Style::default().fg(Color::Cyan)),
                Span::styled(format!("  {}/{}", ch.xp, ch.xp_next), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("   HP ", Style::default().fg(Color::DarkGray)),
                Span::styled(hp_bar, Style::default().fg(hc)),
                Span::styled(format!("  {}/{}", ch.hp, ch.max_hp), Style::default().fg(hc).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("   MP ", Style::default().fg(Color::DarkGray)),
                Span::styled(mp_bar, Style::default().fg(Color::Blue)),
                Span::styled(format!("  {}/{}", ch.mp, ch.max_mp), Style::default().fg(Color::Blue)),
            ]),
            Line::raw(""),
            Line::from(stat_spans),
            Line::raw(""),
            Line::from(vec![
                Span::styled("   Ouro: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<8}", ch.gold), Style::default().fg(Color::Yellow)),
                Span::styled("  Pontos: ", Style::default().fg(Color::DarkGray)),
                Span::styled(ch.stat_points.to_string(), pts_style),
            ]),
            Line::raw(""),
            Line::from(Span::styled(
                "   ↑↓ navegar  [a] alocar  [1] fechar",
                Style::default().fg(Color::DarkGray),
            )),
            Line::raw(""),
        ]
    } else {
        vec![Line::raw("   Carregando...")]
    };

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .title(Span::styled(" ⚔ PERSONAGEM ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        ),
        area,
    );
}

fn item_type_tag(item_type: &ItemType) -> (&'static str, Color) {
    match item_type {
        ItemType::Material   => ("M", Color::DarkGray),
        ItemType::Weapon     => ("W", Color::Yellow),
        ItemType::Armor      => ("A", Color::Blue),
        ItemType::Ring       => ("R", Color::Magenta),
        ItemType::Consumable => ("C", Color::Cyan),
    }
}

fn rarity_color(rarity: &Rarity) -> Color {
    match rarity {
        Rarity::Common   => Color::White,
        Rarity::Uncommon => Color::Green,
        Rarity::Rare     => Color::Magenta,
    }
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
            let is_sel = idx == cursor && focused;
            let prefix = if idx == cursor { "▶" } else { " " };
            let (tag, tag_color) = item_type_tag(&item.item_type);
            let is_equipped = vm.equipped.contains(&item.name);
            let name_color = if is_sel { Color::Yellow } else { rarity_color(&item.rarity) };
            let name_mod = if is_sel { Modifier::BOLD } else { Modifier::empty() };
            let equip_span = if is_equipped {
                Span::styled("[E]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("   ")
            };
            Line::from(vec![
                Span::styled(format!("{} ", prefix), Style::default()),
                Span::styled(format!("[{}]", tag), Style::default().fg(tag_color)),
                equip_span,
                Span::styled(
                    format!(" {} x{}", item.name, item.qty),
                    Style::default().fg(name_color).add_modifier(name_mod),
                ),
            ])
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(focus_style(focused))
                .title("[2] INV  [e] equip  [u] usar"),
        ),
        area,
    );
}

fn map_zone_style(id: &str, vm: &GameViewModel) -> Style {
    let is_current   = vm.zones.iter().any(|z| z.id == id && z.is_current);
    let is_cursor    = vm.cursor_zone_id == Some(id);
    let is_reachable = vm.zones.iter().any(|z| z.id == id && z.is_reachable);
    if is_current {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else if is_cursor {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else if is_reachable {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn map_zone_symbol(id: &str, vm: &GameViewModel) -> &'static str {
    if vm.zones.iter().any(|z| z.id == id && z.is_current) { "● " }
    else if vm.cursor_zone_id == Some(id) { "► " }
    else if vm.zones.iter().any(|z| z.id == id && z.is_reachable) { "○ " }
    else if id == "toca_das_sombras" { "☠ " }
    else if id == "cidade" { "★ " }
    else { "· " }
}

fn render_map_popup(frame: &mut Frame, vm: &GameViewModel) {
    let area = centered_rect(70, 29, frame.area());
    frame.render_widget(Clear, area);

    // Build a zone node span: symbol + name with appropriate style
    let node = |id: &str| -> Span<'static> {
        let name = vm.zones.iter().find(|z| z.id == id).map(|z| z.name).unwrap_or(id);
        let sym  = map_zone_symbol(id, vm);
        Span::styled(format!("{}{}", sym, name), map_zone_style(id, vm))
    };

    let con = Style::default().fg(Color::DarkGray);
    let dim = Style::default().fg(Color::DarkGray);

    let region_sep = |title: &'static str, color: Color| -> Line<'static> {
        Line::from(vec![
            Span::styled("  ── ", con),
            Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(" ─────────────────────────────────────────────────", con),
        ])
    };

    // Current zone info for the footer
    let current = vm.zones.iter().find(|z| z.is_current);
    let cursor_zone = vm.cursor_zone_id
        .and_then(|id| vm.zones.iter().find(|z| z.id == id));

    let footer = if let Some(target) = cursor_zone {
        Line::from(vec![
            Span::styled("  ► ", Style::default().fg(Color::Cyan)),
            Span::styled(target.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("  [Enter] viajar", Style::default().fg(Color::DarkGray)),
            Span::styled("   [m] fechar", Style::default().fg(Color::DarkGray)),
        ])
    } else if let Some(cur) = current {
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::Yellow)),
            Span::styled(cur.name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("  [Enter] viajar   [m] fechar", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(Span::styled("  ↑↓ navegar   Enter: viajar   m: fechar", dim))
    };

    // Layout: 3×3 grid + Cidade (connected to Vila) + Toca das Sombras (connected to Deserto)
    // Center column nodes (Vila/Passagem/Campos) all start at col 19 inside popup content.
    // Vila bullet col = 4 + len("○ Floresta") + len(" ─── ") = 4 + 10 + 5 = 19
    // Deserto bullet col = 4 + len("· Montanha") + len(" ─── ") + len("· Campos") + len(" ─── ") = 4+10+5+8+5 = 32

    let vert19 = || Line::from(vec![Span::raw("                   "), Span::styled("│", con)]);
    let vert32 = || Line::from(vec![Span::raw("                                "), Span::styled("│", con)]);

    let lines: Vec<Line<'_>> = vec![
        Line::raw(""),
        region_sep("REINO DE ALDENMOOR", Color::LightBlue),
        Line::raw(""),
        // Row 1: Floresta ─── Vila ─── Mina
        Line::from(vec![
            Span::raw("    "),
            node("floresta"),
            Span::styled(" ─── ", con),
            node("vila"),
            Span::styled(" ─── ", con),
            node("mina"),
        ]),
        vert19(),
        // Cidade, connected below Vila
        Line::from(vec![
            Span::raw("                   "),
            node("cidade"),
            Span::styled("  ★ cidade segura", Style::default().fg(Color::DarkGray)),
        ]),
        vert19(),
        region_sep("ZONA DE TRANSIÇÃO", Color::LightMagenta),
        // Row 2: Pântano ─── Passagem ─── Caverna
        Line::from(vec![
            Span::raw("    "),
            node("pantano"),
            Span::styled(" ──── ", con),
            node("passagem"),
            Span::styled(" ──── ", con),
            node("caverna"),
        ]),
        vert19(),
        region_sep("TERRAS DO SUL", Color::LightGreen),
        // Row 3: Montanha ─── Campos ─── Deserto
        Line::from(vec![
            Span::raw("    "),
            node("montanha"),
            Span::styled(" ─── ", con),
            node("campos"),
            Span::styled(" ─── ", con),
            node("deserto"),
        ]),
        vert32(),
        region_sep("MASMORRA", Color::Red),
        // Dungeon, connected below Deserto
        Line::from(vec![
            Span::raw("                                "),
            node("toca_das_sombras"),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "  ──────────────────────────────────────────────────────────────",
            con,
        )),
        Line::raw(""),
        footer,
        Line::raw(""),
        Line::from(Span::styled(
            "  ↑↓ selecionar zona",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .title(Span::styled(" ✦ MAPA MUNDIAL ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        ),
        area,
    );
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

fn render_gather_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Gather;

    if let Some(ref progress) = vm.gathering {
        render_gather_progress(frame, area, progress);
        return;
    }

    let actions = &vm.gather_actions;
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
                .border_style(if focused { Style::default().fg(Color::Green) } else { Style::default() })
                .title(Span::styled("[4] COLETA  [4] voltar", Style::default().fg(Color::Green))),
        ),
        area,
    );
}

fn render_gather_progress(frame: &mut Frame, area: Rect, progress: &GatherViewProgress) {
    let bar_width = area.width.saturating_sub(4) as usize;
    let filled = (progress.ratio * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    let lines = vec![
        Line::from(Span::styled(
            format!(" ⟳ {} ({})", progress.action_name, progress.location),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(format!(" {}", bar), Style::default().fg(Color::Green))),
        Line::from(Span::styled(
            format!("  {}/{}s", progress.elapsed_secs, progress.total_secs),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(" [x] cancelar", Style::default().fg(Color::DarkGray))),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(Span::styled("[4] COLETA", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ),
        area,
    );
}

fn render_combat_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Combat;

    if let Some(ref progress) = vm.combat {
        render_combat_progress(frame, area, progress);
        return;
    }

    let mobs = &vm.mobs;
    let h = area.height.saturating_sub(2) as usize;
    let cursor = vm.combat_cursor;
    let start = if cursor >= h { cursor - h + 1 } else { 0 };
    let end = (start + h).min(mobs.len());

    let lines: Vec<Line<'_>> = if mobs.is_empty() {
        vec![Line::raw(" Nenhum inimigo aqui")]
    } else {
        mobs[start..end]
            .iter()
            .enumerate()
            .map(|(i, mob)| {
                let idx = start + i;
                let prefix = if idx == cursor { "▶ " } else { "  " };
                let style = if idx == cursor && focused {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(
                    format!("{}Lv.{} {}  {}s", prefix, mob.level, mob.name, mob.fight_duration_secs),
                    style,
                ))
            })
            .collect()
    };

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused { Style::default().fg(Color::Red) } else { Style::default() })
                .title(Span::styled("[5] COMBATE  [5] voltar", Style::default().fg(Color::Red))),
        ),
        area,
    );
}

fn render_combat_progress(frame: &mut Frame, area: Rect, progress: &CombatViewProgress) {
    let bar_width = area.width.saturating_sub(4) as usize;
    let filled = (progress.ratio * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    let lines = vec![
        Line::from(Span::styled(
            format!(" ⚔ {}", progress.mob_name),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(format!(" {}", bar), Style::default().fg(Color::Red))),
        Line::from(Span::styled(
            format!("  {}/{}s", progress.elapsed_secs, progress.total_secs),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(" [x] cancelar", Style::default().fg(Color::DarkGray))),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(Span::styled("[5] COMBATE", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        ),
        area,
    );
}

fn render_craft_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Craft;
    let total = vm.recipes.len();
    if total == 0 {
        frame.render_widget(
            Paragraph::new("").block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(focus_style(focused))
                    .title("[6] CRAFT"),
            ),
            area,
        );
        return;
    }

    // Split: recipe list on top, ingredient detail below
    let inner_h = area.height.saturating_sub(2) as usize;
    let selected = &vm.recipes[vm.craft_cursor];
    let detail_h = (selected.ingredients.len() + 2).min(inner_h / 2).max(3);
    let list_h = inner_h.saturating_sub(detail_h);

    let cursor = vm.craft_cursor;
    let start = if cursor >= list_h { cursor - list_h + 1 } else { 0 };
    let end = (start + list_h).min(total);

    let mut lines: Vec<Line<'_>> = vm.recipes[start..end]
        .iter()
        .enumerate()
        .map(|(i, recipe)| {
            let idx = start + i;
            let craftable = vm.craftable.get(idx).copied().unwrap_or(false);
            let prefix = if idx == cursor { "▶ " } else { "  " };
            let color = if craftable { Color::Green } else { Color::DarkGray };
            let style = if idx == cursor && focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };
            Line::from(Span::styled(format!("{}{}", prefix, recipe.name), style))
        })
        .collect();

    // Separator
    lines.push(Line::from(Span::styled(
        "─".repeat(area.width.saturating_sub(2) as usize),
        Style::default().fg(Color::DarkGray),
    )));

    // Ingredient detail for selected recipe
    lines.push(Line::from(Span::styled(
        " Ingredientes:",
        Style::default().fg(Color::Gray),
    )));
    for ing in &selected.ingredients {
        let have = vm.inventory.iter()
            .find(|i| i.name == ing.name)
            .map(|i| i.qty)
            .unwrap_or(0);
        let ok = have >= ing.qty;
        let color = if ok { Color::Green } else { Color::Red };
        let mark = if ok { "✓" } else { "✗" };
        lines.push(Line::from(Span::styled(
            format!("  {} x{} ({}/{}) {}", ing.name, ing.qty, have, ing.qty, mark),
            Style::default().fg(color),
        )));
    }

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused { Style::default().fg(Color::Magenta) } else { Style::default() })
                .title(Span::styled("[6] CRAFT  [6] voltar", Style::default().fg(Color::Magenta))),
        ),
        area,
    );
}

fn render_players_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Players;
    let lines: Vec<Line<'_>> = if vm.players_online.is_empty() {
        vec![Line::raw(" Nenhum jogador online")]
    } else {
        vm.players_online.iter()
            .map(|p| Line::from(vec![
                Span::styled(" ● ", Style::default().fg(Color::Green)),
                Span::raw(p.as_str()),
            ]))
            .collect()
    };
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(focus_style(focused))
                .title(Span::styled("[7] ONLINE", Style::default().fg(Color::Green))),
        ),
        area,
    );
}

fn render_npc_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let focused = vm.panel_focus == GamePanel::Npcs;
    let cursor = vm.npc_cursor;
    let lines: Vec<Line<'_>> = if vm.npcs.is_empty() {
        vec![Line::raw(" Nenhum NPC nesta zona")]
    } else {
        vm.npcs.iter().enumerate().map(|(i, npc)| {
            let prefix = if i == cursor { "▶ " } else { "  " };
            let style = if i == cursor && focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(format!("{}★ {}", prefix, npc.name), style))
        }).collect()
    };
    let hint = if vm.npcs.is_empty() { "" } else { "  Enter:falar" };
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() })
                .title(Span::styled(
                    format!("[8] NPCs{}", hint),
                    Style::default().fg(Color::Yellow),
                )),
        ),
        area,
    );
}

fn chat_line_render(text: &str) -> Line<'_> {
    if let Some(colon) = text.find(": ") {
        let nick = &text[..colon];
        let msg  = &text[colon + 2..];
        Line::from(vec![
            Span::styled(nick, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::raw(msg),
        ])
    } else {
        Line::raw(text)
    }
}

fn log_line_style(text: &str) -> Style {
    let t = text.to_lowercase();
    if t.contains("derrotado") || t.contains("foi derrotado") || t.contains("perdeu") {
        Style::default().fg(Color::Red)
    } else if t.contains("subiu para nivel") || t.contains("level up") {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else if t.contains("craftado") || t.contains("coleta") || t.contains("coletar")
           || t.contains("minerar") || t.contains("colher") {
        Style::default().fg(Color::Green)
    } else if t.contains("usada!") || t.contains("hp") && t.contains('+') {
        Style::default().fg(Color::Cyan)
    } else if t.contains("conectado") || t.contains("viajou") {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn render_chat_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let h = area.height.saturating_sub(2) as usize;
    let total = vm.chat_lines.len();
    let offset = vm.chat_scroll.min(total.saturating_sub(h));
    let end = total.saturating_sub(offset);
    let start = end.saturating_sub(h);

    let lines: Vec<Line<'_>> = vm.chat_lines[start..end]
        .iter()
        .map(|l| chat_line_render(l.as_str()))
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL)
                .title(Span::styled("CHAT", Style::default().fg(Color::Cyan))))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_game_log_panel(frame: &mut Frame, area: Rect, vm: &GameViewModel) {
    let h = area.height.saturating_sub(2) as usize;
    let total = vm.game_log.len();
    let end = total;
    let start = end.saturating_sub(h);

    let lines: Vec<Line<'_>> = vm.game_log[start..end]
        .iter()
        .map(|l| Line::from(Span::styled(l.as_str(), log_line_style(l.as_str()))))
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL)
                .title(Span::styled("LOG", Style::default().fg(Color::DarkGray))))
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
